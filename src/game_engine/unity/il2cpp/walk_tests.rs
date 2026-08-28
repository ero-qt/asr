//! Tests pinning the walk's behavior over a hand-laid image of IL2CPP's
//! structures, one fixture per lineage: the older one keeps its metadata
//! handle inline in the image, the newer one behind a pointer. The offsets are
//! the literal numbers of the Unity 2019.4 and 6000.3 layouts, copied by hand,
//! so the walk is checked against the layout rather than against itself.

use super::{IL2CPPOffsets, Module, UnityPointer, Version};
use crate::runtime::mock::with_process;
use crate::{Address, PointerSize, Process};

use std::vec;
use std::vec::Vec;

const BASE: u64 = 0x20_0000;

fn put(image: &mut [u8], at: u64, bytes: &[u8]) {
    let at = at as usize;
    image[at..at + bytes.len()].copy_from_slice(bytes);
}

fn ptr(image: &mut [u8], at: u64, target: u64) {
    put(image, at, &target.to_le_bytes());
}

// The target's structures, hand-laid: the assemblies vector, the type info
// definition table sliced by the image's handle, a parent chain reaching a
// UnityEngine class, a static table, and a live object heading with its class.
fn image(version: Version) -> Vec<u8> {
    let (type_count_at, handle_at, field_count_at) = match version {
        Version::V2019 => (0x1C, 0x18, 0x11C),
        Version::V2020 => (0x18, 0x28, 0x120),
        _ => (0x18, 0x28, 0x124),
    };

    let mut i = vec![0; 0x4000];

    let strings = [
        (0x2000, "mscorlib"),
        (0x2080, "Assembly-CSharp"),
        (0x2100, "GameManager"),
        (0x2180, "Game"),
        (0x2200, "points"),
        (0x2280, "Enemy"),
        (0x2300, "hp"),
        (0x2380, "Boss"),
        (0x2400, "phase"),
        (0x2480, "MonoBehaviour"),
        (0x2500, "UnityEngine"),
        (0x2580, "hidden"),
        (0x2600, "instance"),
        (0x2700, "Outer"),
        (0x2780, "Inner"),
    ];
    for (at, text) in strings {
        put(&mut i, at, text.as_bytes());
    }

    // The assemblies vector: begin and end of an array of assembly pointers.
    ptr(&mut i, 0x0, BASE + 0x40);
    ptr(&mut i, 0x8, BASE + 0x50);
    ptr(&mut i, 0x40, BASE + 0x80);
    ptr(&mut i, 0x48, BASE + 0xC0);

    // Il2CppAssembly: the image at 0x0, the name at 0x18.
    ptr(&mut i, 0x80, BASE + 0x140);
    ptr(&mut i, 0x80 + 0x18, BASE + 0x2000);
    ptr(&mut i, 0xC0, BASE + 0x300);
    ptr(&mut i, 0xC0 + 0x18, BASE + 0x2080);

    // The default image: three classes, reached through the handle. The older
    // lineage stores the handle inline where the newer one points at it.
    put(&mut i, 0x300 + type_count_at, &5_u32.to_le_bytes());
    match version {
        Version::V2019 => put(&mut i, 0x300 + handle_at, &5_u32.to_le_bytes()),
        _ => {
            ptr(&mut i, 0x300 + handle_at, BASE + 0x400);
            put(&mut i, 0x400, &5_u32.to_le_bytes());
        }
    }

    // The type info definition table global, and the image's slice of it.
    ptr(&mut i, 0x10, BASE + 0x480);
    ptr(&mut i, 0x480 + 8 * 5, BASE + 0x600);
    ptr(&mut i, 0x480 + 8 * 6, BASE + 0x800);
    ptr(&mut i, 0x480 + 8 * 7, BASE + 0xA00);
    ptr(&mut i, 0x480 + 8 * 8, BASE + 0x1200);
    ptr(&mut i, 0x480 + 8 * 9, BASE + 0x1400);

    // Il2CppClass: name 0x10, namespace 0x18, parent 0x58, fields 0x80,
    // static_fields 0xB8, field_count where the lineage keeps it. Field
    // entries stride 0x20 with the name at 0x0 and the offset at 0x18.

    // GameManager, deriving from MonoBehaviour, with a static slot and an
    // instance field.
    let game_manager = 0x600;
    ptr(&mut i, game_manager + 0x10, BASE + 0x2100);
    ptr(&mut i, game_manager + 0x18, BASE + 0x2180);
    ptr(&mut i, game_manager + 0x58, BASE + 0xC00);
    ptr(&mut i, game_manager + 0x80, BASE + 0xE00);
    ptr(&mut i, game_manager + 0xB8, BASE + 0xF40);
    put(&mut i, game_manager + field_count_at, &2_u16.to_le_bytes());
    ptr(&mut i, 0xE00, BASE + 0x2600); // instance
    put(&mut i, 0xE00 + 0x18, &0_i32.to_le_bytes());
    ptr(&mut i, 0xE20, BASE + 0x2200); // points
    put(&mut i, 0xE20 + 0x18, &0x20_i32.to_le_bytes());

    // Enemy, and Boss deriving from it.
    let enemy = 0x800;
    ptr(&mut i, enemy + 0x10, BASE + 0x2280);
    ptr(&mut i, enemy + 0x18, BASE + 0x2180);
    ptr(&mut i, enemy + 0x80, BASE + 0xE80);
    put(&mut i, enemy + field_count_at, &1_u16.to_le_bytes());
    ptr(&mut i, 0xE80, BASE + 0x2300); // hp
    put(&mut i, 0xE80 + 0x18, &0x10_i32.to_le_bytes());

    let boss = 0xA00;
    ptr(&mut i, boss + 0x10, BASE + 0x2380);
    ptr(&mut i, boss + 0x18, BASE + 0x2180);
    ptr(&mut i, boss + 0x58, BASE + enemy);
    ptr(&mut i, boss + 0x80, BASE + 0xEC0);
    put(&mut i, boss + field_count_at, &1_u16.to_le_bytes());
    ptr(&mut i, 0xEC0, BASE + 0x2400); // phase
    put(&mut i, 0xEC0 + 0x18, &0x18_i32.to_le_bytes());

    // MonoBehaviour in UnityEngine, holding a field the climb must never
    // reach.
    let mono_behaviour = 0xC00;
    ptr(&mut i, mono_behaviour + 0x10, BASE + 0x2480);
    ptr(&mut i, mono_behaviour + 0x18, BASE + 0x2500);
    ptr(&mut i, mono_behaviour + 0x80, BASE + 0xF00);
    put(
        &mut i,
        mono_behaviour + field_count_at,
        &1_u16.to_le_bytes(),
    );
    ptr(&mut i, 0xF00, BASE + 0x2580); // hidden
    put(&mut i, 0xF00 + 0x18, &0x30_i32.to_le_bytes());

    // Outer in Game, enclosing Inner, whose own namespace is empty and whose
    // declaring type points back out.
    let outer = 0x1200;
    ptr(&mut i, outer + 0x10, BASE + 0x2700);
    ptr(&mut i, outer + 0x18, BASE + 0x2180);
    let inner = 0x1400;
    ptr(&mut i, inner + 0x10, BASE + 0x2780);
    ptr(&mut i, inner + 0x18, BASE + 0x27F0);
    ptr(&mut i, inner + 0x50, BASE + outer);

    // GameManager's statics hold the live instance, which heads with its
    // class.
    ptr(&mut i, 0xF40, BASE + 0xF80);
    ptr(&mut i, 0xF80, BASE + game_manager);
    put(&mut i, 0xF80 + 0x20, &888_u32.to_le_bytes());

    i
}

fn module(version: Version) -> Module {
    Module {
        assemblies: Address::new(BASE),
        type_info_definition_table: Address::new(BASE + 0x10),
        version,
        offsets: IL2CPPOffsets::new(version, PointerSize::Bit64).unwrap(),
        pointer_size: PointerSize::Bit64,
    }
}

fn on_fixture(version: Version, test: impl FnOnce(&Process, &Module)) {
    with_process(&[(BASE, &image(version))], |process| {
        test(process, &module(version));
    });
}

#[test]
fn images_resolve_by_name_in_both_lineages() {
    for version in [Version::V2019, Version::V2022] {
        on_fixture(version, |process, module| {
            assert!(module.get_default_image(process).is_some());
            assert!(module.get_image(process, "mscorlib").is_some());
            assert!(module.get_image(process, "Assembly-DoesNotExist").is_none());
        });
    }
}

#[test]
fn classes_resolve_by_name_and_namespace() {
    for version in [Version::V2019, Version::V2022] {
        on_fixture(version, |process, module| {
            let image = module.get_default_image(process).unwrap();
            assert!(image.get_class(process, module, "GameManager").is_some());
            assert!(image.get_class(process, module, "Game.Boss").is_some());
            assert!(image.get_class(process, module, "Wrong.Boss").is_none());
            assert!(image.get_class(process, module, "Nothing").is_none());
            assert_eq!(image.classes(process, module).count(), 5);
        });
    }
}

#[test]
fn field_offsets_resolve_declared_and_inherited() {
    on_fixture(Version::V2022, |process, module| {
        let image = module.get_default_image(process).unwrap();
        let game_manager = image.get_class(process, module, "GameManager").unwrap();
        assert_eq!(
            game_manager.get_field_offset(process, module, "points"),
            Some(0x20),
        );

        let boss = image.get_class(process, module, "Boss").unwrap();
        assert_eq!(boss.get_field_offset(process, module, "phase"), Some(0x18));
        assert_eq!(boss.get_field_offset(process, module, "hp"), Some(0x10));
    });
}

#[test]
fn nested_classes_resolve_by_their_written_name() {
    on_fixture(Version::V2022, |process, module| {
        let image = module.get_default_image(process).unwrap();
        assert!(image
            .get_class(process, module, "Game.Outer+Inner")
            .is_some());
        assert!(image
            .get_class(process, module, "Game.Outer+Missing")
            .is_none());
        assert!(image
            .get_class(process, module, "Wrong.Outer+Inner")
            .is_none());
        assert!(image
            .get_class(process, module, "Game.Enemy+Inner")
            .is_none());
    });
}

// V2020's table never measured where a class keeps its declaring type, so a
// nested lookup on it must miss cleanly rather than answer with whichever
// class carries the leaf name.
#[test]
fn nested_lookups_without_a_measured_offset_answer_nothing() {
    on_fixture(Version::V2020, |process, module| {
        let image = module.get_default_image(process).unwrap();
        assert!(image
            .get_class(process, module, "Game.Outer+Inner")
            .is_none());
        assert!(image.get_class(process, module, "GameManager").is_some());
    });
}

// The climb stops at UnityEngine's namespace, so an engine field never
// resolves.
#[test]
fn field_climbs_stop_at_the_engine() {
    on_fixture(Version::V2022, |process, module| {
        let image = module.get_default_image(process).unwrap();
        let game_manager = image.get_class(process, module, "GameManager").unwrap();
        assert!(game_manager
            .get_field_offset(process, module, "hidden")
            .is_none());
    });
}

#[test]
fn statics_resolve_from_the_class() {
    on_fixture(Version::V2022, |process, module| {
        let image = module.get_default_image(process).unwrap();
        let game_manager = image.get_class(process, module, "GameManager").unwrap();
        assert_eq!(
            game_manager.get_static_table(process, module),
            Some(Address::new(BASE + 0xF40)),
        );
    });
}

// The whole pointer path: the static root, the instance behind it, and a field
// resolved against the object's own class read off its head.
#[test]
fn pointers_dereference_through_a_static_root() {
    on_fixture(Version::V2022, |process, module| {
        let image = module.get_default_image(process).unwrap();
        let pointer = UnityPointer::<2>::new("GameManager", 0, &["instance", "points"]);
        assert_eq!(pointer.deref::<u32>(process, module, &image).unwrap(), 888);
    });
}

// The public shapes the carve must not change.
#[test]
fn public_types_keep_their_properties() {
    fn is_copy<T: Copy>() {}
    fn double_ended<'a>(
        iter: impl DoubleEndedIterator<Item = super::Class> + 'a,
    ) -> impl DoubleEndedIterator<Item = super::Class> + 'a {
        iter
    }

    is_copy::<super::Image>();
    is_copy::<super::Class>();

    on_fixture(Version::V2022, |process, module| {
        let image = module.get_default_image(process).unwrap();
        let _ = double_ended(image.classes(process, module));
    });
}

// A 32 bit target lays the assemblies vector and its pointers at four bytes.
#[test]
fn images_resolve_on_32_bit_targets() {
    let mut i = vec![0; 0x1000];
    let narrow = |i: &mut [u8], at: u64, target: u64| {
        put(i, at, &(target as u32).to_le_bytes());
    };

    put(&mut i, 0x800, b"Assembly-CSharp");
    narrow(&mut i, 0x0, BASE + 0x40); // the vector's begin
    narrow(&mut i, 0x4, BASE + 0x44); // and end, one assembly along
    narrow(&mut i, 0x40, BASE + 0x80);
    narrow(&mut i, 0x80, BASE + 0x100); // Il2CppAssembly.image
    narrow(&mut i, 0x80 + 0x18, BASE + 0x800); // Il2CppAssembly.aname

    with_process(&[(BASE, &i)], |process| {
        let module = Module {
            assemblies: Address::new(BASE),
            type_info_definition_table: Address::new(BASE + 0x10),
            version: Version::V2022,
            offsets: IL2CPPOffsets::new(Version::V2022, PointerSize::Bit64).unwrap(),
            pointer_size: PointerSize::Bit32,
        };
        assert!(module.get_default_image(process).is_some());
    });
}
