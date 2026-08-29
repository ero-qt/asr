//! Support for attaching to Unity games that are using the IL2CPP backend.

use arrayvec::ArrayVec;
use bytemuck::CheckedBitPattern;

use crate::{
    file_format::pe, future::retry, print_limited, signature::Signature, string::ArrayWString,
    Address, Error, PointerSize, Process,
};

mod builds;
mod image;
pub use image::Image;
mod class;
pub use class::Class;
mod version;
pub use version::Version;
mod pointer;
pub use pointer::UnityPointer;
mod offsets;
use offsets::IL2CPPOffsets;
#[cfg(all(test, not(target_family = "wasm")))]
mod collections_tests;
#[cfg(all(test, not(target_family = "wasm")))]
mod readers_tests;
#[cfg(all(test, not(target_family = "wasm")))]
mod walk_tests;

use super::managed;
pub use super::managed::{DictionaryOffsets, ListOffsets};

/// Represents access to a Unity game that is using the IL2CPP backend.
pub struct Module {
    assemblies: Address,
    type_info_definition_table: Address,
    version: Version,
    offsets: &'static IL2CPPOffsets,
    pointer_size: PointerSize,
}

impl Module {
    /// Tries attaching to a Unity game that is using the IL2CPP backend. If
    /// the game's metadata and Unity versions name a known build, its measured
    /// offsets are used directly. Otherwise this function automatically
    /// detects the [IL2CPP version](Version). If you know the version in
    /// advance or it fails detecting it, use [`attach`](Self::attach) instead.
    pub fn attach_auto_detect(process: &Process) -> Option<Self> {
        let il2cpp_module = Self::find_runtime_module(process)?;
        let pointer_size = pe::MachineType::read(process, il2cpp_module.0)?.pointer_size()?;

        let identity = Self::identity(process);

        if let Some((metadata, unity)) = identity {
            if let Some(build) = builds::find(metadata, unity, pointer_size) {
                if let Some(module) = Self::attach_with(
                    process,
                    il2cpp_module,
                    pointer_size,
                    build.version,
                    &build.offsets,
                ) {
                    print_limited::<128>(&format_args!(
                        "known il2cpp build: metadata {metadata}, unity {}.{}",
                        unity.0, unity.1,
                    ));
                    return Some(module);
                }
            }
        }

        let version = Version::detect(process)?;
        let module = Self::attach(process, version)?;

        match identity {
            Some((metadata, unity)) if builds::find(metadata, unity, pointer_size).is_none() => {
                print_limited::<128>(&format_args!(
                    "unknown il2cpp build: metadata {metadata}, unity {}.{}",
                    unity.0, unity.1,
                ));
            }
            _ => {}
        }

        Some(module)
    }

    /// Tries attaching to a Unity game that is using the IL2CPP backend with
    /// the [IL2CPP version](Version) provided. The version needs to be
    /// correct for this function to work. If you don't know the version in
    /// advance, use [`attach_auto_detect`](Self::attach_auto_detect) instead.
    pub fn attach(process: &Process, version: Version) -> Option<Self> {
        let il2cpp_module = Self::find_runtime_module(process)?;
        let pointer_size = pe::MachineType::read(process, il2cpp_module.0)?.pointer_size()?;
        let offsets = IL2CPPOffsets::new(version, pointer_size)?;

        Self::attach_with(process, il2cpp_module, pointer_size, version, offsets)
    }

    fn find_runtime_module(process: &Process) -> Option<(Address, u64)> {
        let address = process.get_module_address("GameAssembly.dll").ok()?;
        let size = pe::read_size_of_image(process, address)? as u64;
        Some((address, size))
    }

    /// What identifies the game's IL2CPP layout: the version of its mapped
    /// `global-metadata.dat` and the Unity version stamped on the player.
    fn identity(process: &Process) -> Option<(u32, (u16, u16))> {
        let metadata = Self::metadata_version(process)?;

        let unity_player = process.get_module_address("UnityPlayer.dll").ok()?;
        let file_version = pe::FileVersion::read(process, unity_player)?;

        Some((
            metadata,
            (file_version.major_version, file_version.minor_version),
        ))
    }

    /// Reads the version of the game's metadata off the mapped
    /// `global-metadata.dat`, which heads with a sanity value and the version.
    fn metadata_version(process: &Process) -> Option<u32> {
        process.memory_ranges().find_map(|range| {
            let [sanity, version] = process.read::<[u32; 2]>(range.address().ok()?).ok()?;
            // The version numbers run small, the renumbered 6000 line reaching
            // the low hundreds.
            (sanity == 0xFAB1_1BAF && (16..=999).contains(&version)).then_some(version)
        })
    }

    fn attach_with(
        process: &Process,
        il2cpp_module: (Address, u64),
        pointer_size: PointerSize,
        version: Version,
        offsets: &'static IL2CPPOffsets,
    ) -> Option<Self> {
        let assemblies: Address = {
            const ASSEMBLIES: Signature<12> = Signature::new("75 ?? 48 8B 1D ?? ?? ?? ?? 48 3B 1D");
            ASSEMBLIES
                .scan_process_range(process, il2cpp_module)
                .map(|addr| addr + 5)
                .and_then(|addr| Some(addr + 0x4 + process.read::<i32>(addr).ok()?))?
        };

        let type_info_definition_table: Address = {
            const GLOBAL_METADATA: Signature<20> =
                Signature::new("67 6C 6F 62 61 6C 2D 6D 65 74 61 64 61 74 61 2E 64 61 74 00");
            let s_metadata = GLOBAL_METADATA.scan_process_range(process, il2cpp_module)?;

            const LEA: Signature<3> = Signature::new("48 8D 0D");
            let lea: Address = LEA
                .scan_iter(process, il2cpp_module)
                .map(|addr| addr + 3)
                .find(|&addr| {
                    let Ok(offset) = process.read::<i32>(addr) else {
                        return false;
                    };

                    addr + 0x4 + offset == s_metadata
                })?;

            const SHR: Signature<3> = Signature::new("48 C1 E9");
            let shr: Address = SHR
                .scan_process_range(process, (lea, 0x200))
                .map(|addr| addr + 3)?;

            const RAX: Signature<3> = Signature::new("48 89 05");
            RAX.scan_process_range(process, (shr, 0x100))
                .map(|addr| addr + 3)
                .and_then(|addr| Some(addr + 0x4 + process.read::<i32>(addr).ok()?))?
        };

        Some(Self {
            assemblies,
            type_info_definition_table,
            version,
            offsets,
            pointer_size,
        })
    }

    fn walk(&self) -> managed::Walk {
        managed::Walk {
            runtime: managed::Runtime::Il2Cpp(managed::Il2CppRuntime {
                assemblies: self.assemblies,
                type_info_definition_table: self.type_info_definition_table,
                type_count: self.offsets.image.type_count.into(),
                metadata_handle: self.offsets.image.metadata_handle.into(),
                handle_is_inline: matches!(self.version, Version::Base | Version::V2019),
                field_count: self.offsets.class.field_count,
                static_fields: self.offsets.class.static_fields.into(),
                cached_class: self.offsets.generic.cached_class,
                type_data: self.offsets.type_words.data,
                type_kind: self.offsets.type_words.kind,
            }),
            offsets: managed::WalkOffsets {
                assembly: managed::AssemblyOffsets {
                    name_in_image: self.offsets.image.assembly_name.map(u16::from),
                    name_in_assembly: self.offsets.assembly.aname.map(u16::from),
                    image: self.offsets.assembly.image.into(),
                },
                class: managed::ClassOffsets {
                    name: self.offsets.class.name.into(),
                    namespace: self.offsets.class.namespace.into(),
                    parent: self.offsets.class.parent.into(),
                    declaring: self.offsets.class.declaring_type,
                    instance_size: self.offsets.class.instance_size,
                    fields: self.offsets.class.fields.into(),
                },
                field: managed::FieldOffsets {
                    name: self.offsets.field.name.into(),
                    type_: self.offsets.field.type_,
                    offset: self.offsets.field.offset.into(),
                    stride: self.offsets.field.struct_size.into(),
                },
            },
            stop: managed::ClimbStop::UNITY,
            pointer_size: self.pointer_size,
        }
    }

    /// Looks for the specified binary [image](Image) inside the target process.
    /// An [image](Image) is a .NET DLL that is loaded
    /// by the game. The `Assembly-CSharp` [image](Image) is the main game
    /// assembly, and contains all the game logic. The
    /// [`get_default_image`](Self::get_default_image) function is a shorthand
    /// for this function that accesses the `Assembly-CSharp` [image](Image).
    pub fn get_image(&self, process: &Process, assembly_name: &str) -> Option<Image> {
        self.walk()
            .find_image(process, assembly_name)
            .map(|image| Image {
                image: image.address,
            })
    }

    /// Looks for the `Assembly-CSharp` binary [image](Image) inside the target
    /// process. An [image](Image) is a .NET DLL that is loaded
    /// by the game. The `Assembly-CSharp` [image](Image) is the main
    /// game assembly, and contains all the game logic. This function is a
    /// shorthand for [`get_image`](Self::get_image) that accesses the
    /// `Assembly-CSharp` [image](Image).
    pub fn get_default_image(&self, process: &Process) -> Option<Image> {
        self.get_image(process, "Assembly-CSharp")
    }

    /// Reads a managed string through the reference stored at the given
    /// address, such as the end of a pointer path or a slot in a static
    /// table. The string carries its own character count, so no length is
    /// passed; `N` bounds how many UTF-16 characters the returned buffer
    /// holds, and a string claiming more than that fails rather than
    /// truncates, as do a negative count and a null reference. A string
    /// containing an interior nul character reads in full but compares up to
    /// the nul.
    pub fn read_string<const N: usize>(
        &self,
        process: &Process,
        at: Address,
    ) -> Result<ArrayWString<N>, Error> {
        managed::read_string(process, self.pointer_size, at)
    }

    /// Reads a managed array of value elements through the reference stored
    /// at the given address. The array carries its own length, so no count
    /// is passed; `N` bounds how many elements the returned vector holds,
    /// and an array claiming more than that fails rather than truncates, as
    /// does a null reference. The element type is the caller's claim and has
    /// to match the target's own element layout: a managed `char` is a
    /// `u16` here, a `bool` a single byte, and Rust's `char` and `usize`
    /// never match. Reference elements have no portable claim.
    pub fn read_array<T: CheckedBitPattern, const N: usize>(
        &self,
        process: &Process,
        at: Address,
    ) -> Result<ArrayVec<T, N>, Error> {
        managed::read_array(process, self.pointer_size, at)
    }

    /// Resolves where a `List` keeps its backing array and live count, off
    /// the class the list object at the given address names as its own. The
    /// answer is a small `Copy` value worth storing, like a field offset:
    /// resolution walks the class's fields, where the read itself is a
    /// handful of reads. An object whose class is not a list misses.
    pub fn get_list_offsets(&self, process: &Process, at: Address) -> Option<ListOffsets> {
        let object = process
            .read_pointer(at, self.pointer_size)
            .ok()
            .filter(|address| !address.is_null())?;

        self.walk().list_offsets(process, object)
    }

    /// Resolves where a `Dictionary` keeps its backing entries and live
    /// counts, and how one entry lays out, off the class the dictionary
    /// object at the given address names as its own. The answer is a small
    /// `Copy` value worth storing, like a field offset: resolution walks
    /// class metadata, where the read itself is a handful of reads. An
    /// object whose class is not this dictionary shape, and a target still
    /// starting up, both miss.
    pub fn get_dictionary_offsets(
        &self,
        process: &Process,
        at: Address,
    ) -> Option<DictionaryOffsets> {
        let object = process
            .read_pointer(at, self.pointer_size)
            .ok()
            .filter(|address| !address.is_null())?;

        self.walk().dictionary_offsets(process, object)
    }

    /// Returns the pointer size the target runs at, which is what a caller
    /// claims reference-width values with.
    pub fn get_pointer_size(&self) -> PointerSize {
        self.pointer_size
    }

    /// Reads a managed `List` of value elements through the reference stored
    /// at the given address, with the offsets
    /// [`get_list_offsets`](Self::get_list_offsets) resolved. The list's
    /// live count is read, never its backing capacity; `N` bounds the
    /// count, and a count past the buffer or past the backing array's own
    /// length fails rather than truncates, as does a null reference. The
    /// element type is the caller's claim, as with
    /// [`read_array`](Self::read_array).
    pub fn read_list<T: CheckedBitPattern, const N: usize>(
        &self,
        process: &Process,
        offsets: ListOffsets,
        at: Address,
    ) -> Result<ArrayVec<T, N>, Error> {
        managed::read_list(process, self.pointer_size, offsets, at)
    }

    /// Attaches to a Unity game that is using the IL2CPP backend. This function
    /// automatically detects the [IL2CPP version](Version). If you know the
    /// version in advance or it fails detecting it, use
    /// [`wait_attach`](Self::wait_attach) instead.
    ///
    /// This is the `await`able version of the
    /// [`attach_auto_detect`](Self::attach_auto_detect) function, yielding back
    /// to the runtime between each try.
    pub async fn wait_attach_auto_detect(process: &Process) -> Module {
        retry(|| Self::attach_auto_detect(process)).await
    }

    /// Attaches to a Unity game that is using the IL2CPP backend with the
    /// [IL2CPP version](Version) provided. The version needs to be correct
    /// for this function to work. If you don't know the version in advance, use
    /// [`wait_attach_auto_detect`](Self::wait_attach_auto_detect) instead.
    ///
    /// This is the `await`able version of the [`attach`](Self::attach)
    /// function, yielding back to the runtime between each try.
    pub async fn wait_attach(process: &Process, version: Version) -> Module {
        retry(|| Self::attach(process, version)).await
    }

    /// Looks for the specified binary [image](Image) inside the target process.
    /// An [image](Image) is a .NET DLL that is loaded
    /// by the game. The `Assembly-CSharp` [image](Image) is the main game
    /// assembly, and contains all the game logic. The
    /// [`wait_get_default_image`](Self::wait_get_default_image) function is a
    /// shorthand for this function that accesses the `Assembly-CSharp`
    /// [image](Image).
    ///
    /// This is the `await`able version of the [`get_image`](Self::get_image)
    /// function, yielding back to the runtime between each try.
    pub async fn wait_get_image(&self, process: &Process, assembly_name: &str) -> Image {
        retry(|| self.get_image(process, assembly_name)).await
    }

    /// Looks for the `Assembly-CSharp` binary [image](Image) inside the target
    /// process. An [image](Image) is a .NET DLL that
    /// is loaded by the game. The `Assembly-CSharp` [image](Image) is the main
    /// game assembly, and contains all the game logic. This function is a
    /// shorthand for [`wait_get_image`](Self::wait_get_image) that accesses the
    /// `Assembly-CSharp` [image](Image).
    ///
    /// This is the `await`able version of the
    /// [`get_default_image`](Self::get_default_image) function, yielding back
    /// to the runtime between each try.
    pub async fn wait_get_default_image(&self, process: &Process) -> Image {
        retry(|| self.get_default_image(process)).await
    }

    /// Resolves where a `List` keeps its backing array and live count, off
    /// the class the list object at the given address names as its own.
    ///
    /// This is the `await`able version of the
    /// [`get_list_offsets`](Self::get_list_offsets) function, yielding back
    /// to the runtime between each try.
    pub async fn wait_get_list_offsets(&self, process: &Process, at: Address) -> ListOffsets {
        retry(|| self.get_list_offsets(process, at)).await
    }

    /// Resolves where a `Dictionary` keeps its backing entries and live
    /// counts, and how one entry lays out, off the class the dictionary
    /// object at the given address names as its own.
    ///
    /// This is the `await`able version of the
    /// [`get_dictionary_offsets`](Self::get_dictionary_offsets) function,
    /// yielding back to the runtime between each try.
    pub async fn wait_get_dictionary_offsets(
        &self,
        process: &Process,
        at: Address,
    ) -> DictionaryOffsets {
        retry(|| self.get_dictionary_offsets(process, at)).await
    }
}

#[cfg(all(test, not(target_family = "wasm")))]
mod tests {
    use super::Module;
    use crate::runtime::mock::with_process;

    #[test]
    fn reads_the_metadata_version_off_the_mapped_file() {
        let mapped = [0xAF_u8, 0x1B, 0xB1, 0xFA, 39, 0, 0, 0];
        // The sanity value with nothing sane behind it must not answer.
        let stray = [0xAF_u8, 0x1B, 0xB1, 0xFA, 0, 0, 0, 0];

        with_process(&[(0x10000, &stray), (0x20000, &mapped)], |process| {
            assert_eq!(Module::metadata_version(process), Some(39));
        });

        with_process(&[(0x10000, &stray)], |process| {
            assert!(Module::metadata_version(process).is_none());
        });
    }
}
