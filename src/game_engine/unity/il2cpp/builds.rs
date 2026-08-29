//! Known IL2CPP builds: exact metadata layouts, named by the version of the
//! game's `global-metadata.dat` and the Unity version that shipped it, paired
//! with the offsets measured from their symbols.

use super::offsets::{
    AssemblyOffsets, ClassOffsets, FieldInfoOffsets, GenericOffsets, IL2CPPOffsets, ImageOffsets,
    TypeOffsets,
};
use super::Version;
use crate::PointerSize;

/// One exact IL2CPP layout and the offsets measured from it.
pub(super) struct Build {
    pub(super) metadata: u32,
    pub(super) unity: (u16, u16),
    pub(super) pointer_size: PointerSize,
    pub(super) version: Version,
    pub(super) offsets: IL2CPPOffsets,
}

/// Looks up the newest known build at or below the given identity. Unlike a
/// mono runtime, `GameAssembly.dll` is compiled per game, so no identity names
/// one binary: a build declares the version it applies from, and an identity
/// below the oldest known build answers nothing.
pub(super) fn find(
    metadata: u32,
    unity: (u16, u16),
    pointer_size: PointerSize,
) -> Option<&'static Build> {
    BUILDS
        .iter()
        .rev()
        .filter(|build| build.pointer_size == pointer_size)
        .find(|build| (build.metadata, build.unity) <= (metadata, unity))
}

// The table reads from the oldest metadata to the newest.
static BUILDS: &[Build] = &[
    // Unity 2018.4.36f1, metadata version 24, x64.
    // Offsets from the player's own GameAssembly pdb, scans matched against the symbols they resolve to.
    // The class has no unity_user_data yet, so everything past it sits eight bytes lower than in 2019.4.
    Build {
        metadata: 24,
        unity: (2018, 4),
        pointer_size: PointerSize::Bit64,
        version: Version::Base,
        offsets: IL2CPPOffsets {
            assembly: AssemblyOffsets {
                image: 0x0,
                aname: None,
            },
            image: ImageOffsets {
                assembly_name: Some(0x8),
                type_count: 0x1c,
                metadata_handle: 0x18,
            },
            class: ClassOffsets {
                name: 0x10,
                namespace: 0x18,
                declaring_type: Some(0x50),
                parent: 0x58,
                fields: 0x80,
                static_fields: 0xb8,
                instance_size: Some(0xec),
                field_count: 0x114,
            },
            generic: GenericOffsets {
                cached_class: Some(0x18),
            },
            type_words: TypeOffsets {
                data: Some(0x0),
                kind: Some(0xa),
            },
            field: FieldInfoOffsets {
                name: 0x0,
                type_: Some(0x8),
                offset: 0x18,
                struct_size: 0x20,
            },
        },
    },
    // Unity 2019.4.41f2 and 2020.1.18f1, metadata version 24, x64.
    // Offsets from each player's own GameAssembly pdb, scans matched against the symbols they resolve to.
    Build {
        metadata: 24,
        unity: (2019, 4),
        pointer_size: PointerSize::Bit64,
        version: Version::V2019,
        offsets: IL2CPPOffsets {
            assembly: AssemblyOffsets {
                image: 0x0,
                aname: None,
            },
            image: ImageOffsets {
                assembly_name: Some(0x8),
                type_count: 0x1c,
                metadata_handle: 0x18,
            },
            class: ClassOffsets {
                name: 0x10,
                namespace: 0x18,
                declaring_type: Some(0x50),
                parent: 0x58,
                fields: 0x80,
                static_fields: 0xb8,
                instance_size: Some(0xf4),
                field_count: 0x11c,
            },
            generic: GenericOffsets {
                cached_class: Some(0x18),
            },
            type_words: TypeOffsets {
                data: Some(0x0),
                kind: Some(0xa),
            },
            field: FieldInfoOffsets {
                name: 0x0,
                type_: Some(0x8),
                offset: 0x18,
                struct_size: 0x20,
            },
        },
    },
    // Unity 2020.1.18, metadata version 24, x64, measured at release and master.
    Build {
        metadata: 24,
        unity: (2020, 1),
        pointer_size: PointerSize::Bit64,
        version: Version::V2019,
        offsets: IL2CPPOffsets {
            assembly: AssemblyOffsets {
                image: 0x0,
                aname: None,
            },
            image: ImageOffsets {
                assembly_name: Some(0x8),
                type_count: 0x1c,
                metadata_handle: 0x18,
            },
            class: ClassOffsets {
                name: 0x10,
                namespace: 0x18,
                declaring_type: Some(0x50),
                parent: 0x58,
                fields: 0x80,
                static_fields: 0xb8,
                instance_size: Some(0xf4),
                field_count: 0x11c,
            },
            generic: GenericOffsets {
                cached_class: Some(0x18),
            },
            type_words: TypeOffsets {
                data: Some(0x0),
                kind: Some(0xa),
            },
            field: FieldInfoOffsets {
                name: 0x0,
                type_: Some(0x8),
                offset: 0x18,
                struct_size: 0x20,
            },
        },
    },
    // Unity 2020.1.18, metadata version 24, x86, measured at release and master.
    Build {
        metadata: 24,
        unity: (2020, 1),
        pointer_size: PointerSize::Bit32,
        version: Version::V2019,
        offsets: IL2CPPOffsets {
            assembly: AssemblyOffsets {
                image: 0x0,
                aname: None,
            },
            image: ImageOffsets {
                assembly_name: Some(0x4),
                type_count: 0x10,
                metadata_handle: 0xc,
            },
            class: ClassOffsets {
                name: 0x8,
                namespace: 0xc,
                declaring_type: Some(0x28),
                parent: 0x2c,
                fields: 0x40,
                static_fields: 0x5c,
                instance_size: Some(0x80),
                field_count: 0xa8,
            },
            generic: GenericOffsets {
                cached_class: Some(0xc),
            },
            type_words: TypeOffsets {
                data: Some(0x0),
                kind: Some(0x6),
            },
            field: FieldInfoOffsets {
                name: 0x0,
                type_: Some(0x4),
                offset: 0xc,
                struct_size: 0x14,
            },
        },
    },
    // Unity 2021.3.11f1, metadata version 29, x64.
    // Offsets from the player's own GameAssembly pdb, scans matched against the symbols they resolve to.
    // Metadata 29 spans 2021.3 to 2023.1 and the class is not the same across it: 2023.1 inserted
    // stack_slot_size after instance_size, which puts field_count at 0x120 here and 0x124 there. Every
    // other offset the walk reads is identical, and neither 2023.1's assemblies scan nor its type table
    // scan reaches its symbol on these binaries, so this version carries a set of its own.
    Build {
        metadata: 29,
        unity: (2021, 3),
        pointer_size: PointerSize::Bit64,
        version: Version::V2020,
        offsets: IL2CPPOffsets {
            assembly: AssemblyOffsets {
                image: 0x0,
                aname: None,
            },
            image: ImageOffsets {
                assembly_name: Some(0x8),
                type_count: 0x18,
                metadata_handle: 0x28,
            },
            class: ClassOffsets {
                name: 0x10,
                namespace: 0x18,
                declaring_type: Some(0x50),
                parent: 0x58,
                fields: 0x80,
                static_fields: 0xb8,
                instance_size: Some(0xf8),
                field_count: 0x120,
            },
            generic: GenericOffsets {
                cached_class: Some(0x18),
            },
            type_words: TypeOffsets {
                data: Some(0x0),
                kind: Some(0xa),
            },
            field: FieldInfoOffsets {
                name: 0x0,
                type_: Some(0x8),
                offset: 0x18,
                struct_size: 0x20,
            },
        },
    },
    // Unity 2021.3.11f1, metadata version 29, x86. Verified live against the fixture player.
    // Offsets from the player's own GameAssembly pdb, scans matched against the symbols they resolve to.
    // x86 code names a global outright rather than by a displacement, so every scan resolves absolute.
    // Metadata 29 reaches 2023.1, which moves field_count a word on and carries an entry of its own.
    Build {
        metadata: 29,
        unity: (2021, 3),
        pointer_size: PointerSize::Bit32,
        version: Version::V2020,
        offsets: IL2CPPOffsets {
            assembly: AssemblyOffsets {
                image: 0x0,
                aname: None,
            },
            image: ImageOffsets {
                assembly_name: Some(0x4),
                type_count: 0xc,
                metadata_handle: 0x18,
            },
            class: ClassOffsets {
                name: 0x8,
                namespace: 0xc,
                declaring_type: Some(0x28),
                parent: 0x2c,
                fields: 0x40,
                static_fields: 0x5c,
                instance_size: Some(0x80),
                field_count: 0xa8,
            },
            generic: GenericOffsets {
                cached_class: Some(0xc),
            },
            type_words: TypeOffsets {
                data: Some(0x0),
                kind: Some(0x6),
            },
            field: FieldInfoOffsets {
                name: 0x0,
                type_: Some(0x4),
                offset: 0xc,
                struct_size: 0x14,
            },
        },
    },
    // Unity 2023.1.22f1, metadata version 29, x64.
    // Offsets from the player's own GameAssembly pdb, scans matched against the symbols they resolve to.
    Build {
        metadata: 29,
        unity: (2023, 1),
        pointer_size: PointerSize::Bit64,
        version: Version::V2022,
        offsets: IL2CPPOffsets {
            assembly: AssemblyOffsets {
                image: 0x0,
                aname: None,
            },
            image: ImageOffsets {
                assembly_name: Some(0x8),
                type_count: 0x18,
                metadata_handle: 0x28,
            },
            class: ClassOffsets {
                name: 0x10,
                namespace: 0x18,
                declaring_type: Some(0x50),
                parent: 0x58,
                fields: 0x80,
                static_fields: 0xb8,
                instance_size: Some(0xf8),
                field_count: 0x124,
            },
            generic: GenericOffsets {
                cached_class: Some(0x18),
            },
            type_words: TypeOffsets {
                data: Some(0x0),
                kind: Some(0xa),
            },
            field: FieldInfoOffsets {
                name: 0x0,
                type_: Some(0x8),
                offset: 0x18,
                struct_size: 0x20,
            },
        },
    },
    // Unity 2023.1.22f1, metadata version 29, x86. Verified live against the fixture player.
    // Offsets from the player's own GameAssembly pdb, scans matched against the symbols they resolve to.
    // x86 code names a global outright rather than by a displacement, so every scan resolves absolute.
    // Metadata 29 spans 2021.3 to 2023.1 and the class is not the same across it, at either width:
    // 2023.1 inserted stack_slot_size after instance_size, which puts field_count at 0xA8 on 2021.3 and
    // 0xAC here, the same one word move the 64 bit pair carries at 0x120 and 0x124. Every other offset
    // the walk reads is identical, measured member for member off both players' own pdbs, and every
    // scan below reaches its global on this version unchanged, which is what lets this entry be the
    // 2021.3 one with a single number moved rather than a set of its own.
    Build {
        metadata: 29,
        unity: (2023, 1),
        pointer_size: PointerSize::Bit32,
        version: Version::V2022,
        offsets: IL2CPPOffsets {
            assembly: AssemblyOffsets {
                image: 0x0,
                aname: None,
            },
            image: ImageOffsets {
                assembly_name: Some(0x4),
                type_count: 0xc,
                metadata_handle: 0x18,
            },
            class: ClassOffsets {
                name: 0x8,
                namespace: 0xc,
                declaring_type: Some(0x28),
                parent: 0x2c,
                fields: 0x40,
                static_fields: 0x5c,
                instance_size: Some(0x80),
                field_count: 0xac,
            },
            generic: GenericOffsets {
                cached_class: Some(0xc),
            },
            type_words: TypeOffsets {
                data: Some(0x0),
                kind: Some(0x6),
            },
            field: FieldInfoOffsets {
                name: 0x0,
                type_: Some(0x4),
                offset: 0xc,
                struct_size: 0x14,
            },
        },
    },
    // Unity 6000.2.12, metadata version 31, x64, measured at master and release.
    Build {
        metadata: 31,
        unity: (6000, 2),
        pointer_size: PointerSize::Bit64,
        version: Version::V2022,
        offsets: IL2CPPOffsets {
            assembly: AssemblyOffsets {
                image: 0x0,
                aname: None,
            },
            image: ImageOffsets {
                assembly_name: Some(0x8),
                type_count: 0x18,
                metadata_handle: 0x28,
            },
            class: ClassOffsets {
                name: 0x10,
                namespace: 0x18,
                declaring_type: Some(0x50),
                parent: 0x58,
                fields: 0x80,
                static_fields: 0xb8,
                instance_size: Some(0xf8),
                field_count: 0x124,
            },
            generic: GenericOffsets {
                cached_class: Some(0x18),
            },
            type_words: TypeOffsets {
                data: Some(0x0),
                kind: Some(0xa),
            },
            field: FieldInfoOffsets {
                name: 0x0,
                type_: Some(0x8),
                offset: 0x18,
                struct_size: 0x20,
            },
        },
    },
    // Unity 6000.2.12, metadata version 31, x86, measured at release and master.
    Build {
        metadata: 31,
        unity: (6000, 2),
        pointer_size: PointerSize::Bit32,
        version: Version::V2022,
        offsets: IL2CPPOffsets {
            assembly: AssemblyOffsets {
                image: 0x0,
                aname: None,
            },
            image: ImageOffsets {
                assembly_name: Some(0x4),
                type_count: 0xc,
                metadata_handle: 0x18,
            },
            class: ClassOffsets {
                name: 0x8,
                namespace: 0xc,
                declaring_type: Some(0x28),
                parent: 0x2c,
                fields: 0x40,
                static_fields: 0x5c,
                instance_size: Some(0x80),
                field_count: 0xac,
            },
            generic: GenericOffsets {
                cached_class: Some(0xc),
            },
            type_words: TypeOffsets {
                data: Some(0x0),
                kind: Some(0x6),
            },
            field: FieldInfoOffsets {
                name: 0x0,
                type_: Some(0x4),
                offset: 0xc,
                struct_size: 0x14,
            },
        },
    },
    // Unity 6000.3.21f1, metadata version 39, x64.
    // Offsets from the player's own GameAssembly pdb, scans matched against the symbols they resolve to.
    // Unity 6 numbers its metadata apart from what came before: 6000.3 is 39 where 2023.1 was 29, and
    // 6000.5 is 107. The layout does not follow that numbering, and this one is 2023.1's rather than
    // 6000.5's, `static_fields` having moved to 0xA0 only in the later one.
    Build {
        metadata: 39,
        unity: (6000, 3),
        pointer_size: PointerSize::Bit64,
        version: Version::V2022,
        offsets: IL2CPPOffsets {
            assembly: AssemblyOffsets {
                image: 0x0,
                aname: None,
            },
            image: ImageOffsets {
                assembly_name: Some(0x8),
                type_count: 0x18,
                metadata_handle: 0x28,
            },
            class: ClassOffsets {
                name: 0x10,
                namespace: 0x18,
                declaring_type: Some(0x50),
                parent: 0x58,
                fields: 0x80,
                static_fields: 0xb8,
                instance_size: Some(0xf8),
                field_count: 0x124,
            },
            generic: GenericOffsets {
                cached_class: Some(0x18),
            },
            type_words: TypeOffsets {
                data: Some(0x0),
                kind: Some(0xa),
            },
            field: FieldInfoOffsets {
                name: 0x0,
                type_: Some(0x8),
                offset: 0x18,
                struct_size: 0x20,
            },
        },
    },
    // Unity 6000.3.21f1, metadata version 39, x86.
    // Offsets from the player's own GameAssembly pdb, scans matched against the symbols they resolve to
    // and held to both configurations of the 32 bit player.
    Build {
        metadata: 39,
        unity: (6000, 3),
        pointer_size: PointerSize::Bit32,
        version: Version::V2022,
        offsets: IL2CPPOffsets {
            assembly: AssemblyOffsets {
                image: 0x0,
                aname: None,
            },
            image: ImageOffsets {
                assembly_name: Some(0x4),
                type_count: 0xc,
                metadata_handle: 0x18,
            },
            class: ClassOffsets {
                name: 0x8,
                namespace: 0xc,
                declaring_type: Some(0x28),
                parent: 0x2c,
                fields: 0x40,
                static_fields: 0x5c,
                instance_size: Some(0x80),
                field_count: 0xac,
            },
            generic: GenericOffsets {
                cached_class: Some(0xc),
            },
            type_words: TypeOffsets {
                data: Some(0x0),
                kind: Some(0x6),
            },
            field: FieldInfoOffsets {
                name: 0x0,
                type_: Some(0x4),
                offset: 0xc,
                struct_size: 0x14,
            },
        },
    },
    // Unity 6000.5.8f1, metadata version 107, x64.
    // Offsets from the editor's own libil2cpp pdb, scans matched against the symbols they resolve to.
    Build {
        metadata: 107,
        unity: (6000, 5),
        pointer_size: PointerSize::Bit64,
        version: Version::V2022,
        offsets: IL2CPPOffsets {
            assembly: AssemblyOffsets {
                image: 0x0,
                aname: None,
            },
            image: ImageOffsets {
                assembly_name: Some(0x8),
                type_count: 0x18,
                metadata_handle: 0x28,
            },
            class: ClassOffsets {
                name: 0x10,
                namespace: 0x18,
                declaring_type: Some(0x50),
                parent: 0x58,
                fields: 0x80,
                static_fields: 0xa0,
                instance_size: Some(0xf8),
                field_count: 0x124,
            },
            generic: GenericOffsets {
                cached_class: Some(0x18),
            },
            type_words: TypeOffsets {
                data: Some(0x0),
                kind: Some(0xa),
            },
            field: FieldInfoOffsets {
                name: 0x0,
                type_: Some(0x8),
                offset: 0x18,
                struct_size: 0x20,
            },
        },
    },
    // Unity 6000.5.8f1, metadata version 107, x86.
    // Offsets from the player's own GameAssembly pdb, scans matched against the symbols they resolve to
    // and held to both configurations of the 32 bit player.
    // Neither neighbour narrowed: the assembly stride is 110's where the generic class is 39's, and
    // `static_fields` sits between the two at 0x50, so this width was measured rather than inferred.
    Build {
        metadata: 107,
        unity: (6000, 5),
        pointer_size: PointerSize::Bit32,
        version: Version::V2022,
        offsets: IL2CPPOffsets {
            assembly: AssemblyOffsets {
                image: 0x0,
                aname: None,
            },
            image: ImageOffsets {
                assembly_name: Some(0x4),
                type_count: 0xc,
                metadata_handle: 0x18,
            },
            class: ClassOffsets {
                name: 0x8,
                namespace: 0xc,
                declaring_type: Some(0x28),
                parent: 0x2c,
                fields: 0x40,
                static_fields: 0x50,
                instance_size: Some(0x80),
                field_count: 0xac,
            },
            generic: GenericOffsets {
                cached_class: Some(0xc),
            },
            type_words: TypeOffsets {
                data: Some(0x0),
                kind: Some(0x6),
            },
            field: FieldInfoOffsets {
                name: 0x0,
                type_: Some(0x4),
                offset: 0xc,
                struct_size: 0x14,
            },
        },
    },
    // Unity 6000.7.0a3, metadata version 110, x64.
    // Offsets from the player's own GameAssembly pdb, cross checked against the editor's own
    // libil2cpp headers, which agree member for member. Four of them differ from 6000.3's, that era
    // carrying members this one has dropped, so nothing here is that entry carried forward.
    Build {
        metadata: 110,
        unity: (6000, 7),
        pointer_size: PointerSize::Bit64,
        version: Version::V2022,
        offsets: IL2CPPOffsets {
            assembly: AssemblyOffsets {
                image: 0x0,
                aname: None,
            },
            image: ImageOffsets {
                assembly_name: Some(0x8),
                type_count: 0x18,
                metadata_handle: 0x28,
            },
            class: ClassOffsets {
                name: 0x10,
                namespace: 0x18,
                declaring_type: Some(0x50),
                parent: 0x58,
                fields: 0x80,
                static_fields: 0x98,
                instance_size: Some(0xf0),
                field_count: 0x11c,
            },
            generic: GenericOffsets {
                cached_class: Some(0x10),
            },
            type_words: TypeOffsets {
                data: Some(0x0),
                kind: Some(0xa),
            },
            field: FieldInfoOffsets {
                name: 0x0,
                type_: Some(0x8),
                offset: 0x18,
                struct_size: 0x20,
            },
        },
    },
    // Unity 6000.7.0a3, metadata version 110, x86.
    // Offsets from the player's own GameAssembly pdb. Three of them differ from 6000.3's at this width,
    // the same three that differ at 64 bits, so this is measured rather than that entry narrowed:
    // `static_fields` sits earlier, the assembly stride is wider, and the generic class is shorter.
    // A Master player exists at this width too, and of the three roots only the assemblies table is
    // reached differently there, so that one alone carries a Master shape beside its release ones.
    Build {
        metadata: 110,
        unity: (6000, 7),
        pointer_size: PointerSize::Bit32,
        version: Version::V2022,
        offsets: IL2CPPOffsets {
            assembly: AssemblyOffsets {
                image: 0x0,
                aname: None,
            },
            image: ImageOffsets {
                assembly_name: Some(0x4),
                type_count: 0xc,
                metadata_handle: 0x18,
            },
            class: ClassOffsets {
                name: 0x8,
                namespace: 0xc,
                declaring_type: Some(0x28),
                parent: 0x2c,
                fields: 0x40,
                static_fields: 0x4c,
                instance_size: Some(0x80),
                field_count: 0xac,
            },
            generic: GenericOffsets {
                cached_class: Some(0x8),
            },
            type_words: TypeOffsets {
                data: Some(0x0),
                kind: Some(0x6),
            },
            field: FieldInfoOffsets {
                name: 0x0,
                type_: Some(0x4),
                offset: 0xc,
                struct_size: 0x14,
            },
        },
    },
];

#[cfg(all(test, not(target_family = "wasm")))]
mod tests {
    use super::super::offsets::IL2CPPOffsets;
    use super::super::Version;
    use super::{find, BUILDS};
    use crate::PointerSize;

    #[test]
    fn table_reads_oldest_to_newest() {
        assert!(BUILDS
            .windows(2)
            .all(|pair| (pair[0].metadata, pair[0].unity) <= (pair[1].metadata, pair[1].unity)));
    }

    #[test]
    fn finds_exact_builds() {
        let build = find(39, (6000, 3), PointerSize::Bit64).unwrap();
        assert_eq!(build.metadata, 39);
        assert_eq!(build.unity, (6000, 3));

        let narrow = find(39, (6000, 3), PointerSize::Bit32).unwrap();
        assert_eq!(narrow.metadata, 39);
        assert_eq!(narrow.pointer_size, PointerSize::Bit32);
    }

    #[test]
    fn unmeasured_identities_answer_the_newest_build_below() {
        let build = find(29, (2022, 1), PointerSize::Bit64).unwrap();
        assert_eq!((build.metadata, build.unity), (29, (2021, 3)));

        let build = find(35, (6000, 0), PointerSize::Bit64).unwrap();
        assert_eq!((build.metadata, build.unity), (31, (6000, 2)));

        let build = find(200, (7000, 0), PointerSize::Bit64).unwrap();
        assert_eq!((build.metadata, build.unity), (110, (6000, 7)));
    }

    #[test]
    fn identities_below_the_oldest_build_answer_nothing() {
        assert!(find(16, (5, 6), PointerSize::Bit64).is_none());
        assert!(find(24, (2018, 4), PointerSize::Bit32).is_none());
    }

    // A version table's value for any of the grown members must match every
    // measured build it stands in for, or say nothing.
    #[test]
    fn version_tables_never_contradict_a_measured_build() {
        fn agrees(table: Option<u16>, measured: Option<u16>) -> bool {
            table.is_none() || table == measured
        }

        for build in BUILDS {
            let Some(table) = IL2CPPOffsets::new(build.version, build.pointer_size) else {
                continue;
            };
            assert!(agrees(
                table.class.declaring_type,
                build.offsets.class.declaring_type
            ));
            assert!(agrees(
                table.class.instance_size,
                build.offsets.class.instance_size
            ));
            assert!(agrees(
                table.generic.cached_class,
                build.offsets.generic.cached_class
            ));
            assert!(agrees(table.type_words.data, build.offsets.type_words.data));
            assert!(agrees(table.type_words.kind, build.offsets.type_words.kind));
            assert!(agrees(table.field.type_, build.offsets.field.type_));
        }
    }

    // The shipped table for 2022.2 and later keeps static_fields at 0xB8;
    // 6000.5 measures 0xA0 and 6000.7 measures 0x98 with a smaller
    // field_count. The entries keep what was measured.
    #[test]
    fn unity_6000_5_builds_diverge_from_their_version_table_on_statics() {
        let build = find(107, (6000, 5), PointerSize::Bit64).unwrap();
        assert!(matches!(build.version, Version::V2022));
        assert_eq!(build.offsets.class.static_fields, 0xA0);

        let build = find(110, (6000, 7), PointerSize::Bit64).unwrap();
        assert_eq!(build.offsets.class.static_fields, 0x98);
        assert_eq!(build.offsets.class.field_count, 0x11C);
    }
}
