//! Complete Unreal Engine layouts for 64-bit Windows games.
//!
//! Each constant is the layout from the engine version in its name on. The
//! layout holds until the version of the next constant.

use super::{ClassOffsets, ObjectOffsets, Profile, PropertyOffsets, FNAME_POOL, G_ENGINE, G_WORLD};
use crate::PointerSize;

// Tested on: Sonic Omens
/// Unreal Engine 4.23, x86-64.
pub const UNREAL_4_23_X86_64: Profile = Profile {
    pointer_size: PointerSize::Bit64,
    g_engine: G_ENGINE[0],
    g_world: G_WORLD[0],
    fname_pool: FNAME_POOL[0],
    object: ObjectOffsets {
        fname: 0x18,
        class: 0x10,
    },
    class: ClassOffsets {
        super_field: 0x40,
        property_link: 0x48,
    },
    property: PropertyOffsets {
        fname: 0x18,
        offset_internal: 0x44,
        property_link_next: 0x50,
    },
};

// Tested on: Tetris Effect / Kao the Kangaroo
/// Unreal Engine 4.25, x86-64.
pub const UNREAL_4_25_X86_64: Profile = Profile {
    pointer_size: PointerSize::Bit64,
    g_engine: G_ENGINE[0],
    g_world: G_WORLD[0],
    fname_pool: FNAME_POOL[0],
    object: ObjectOffsets {
        fname: 0x18,
        class: 0x10,
    },
    class: ClassOffsets {
        super_field: 0x40,
        property_link: 0x50,
    },
    property: PropertyOffsets {
        fname: 0x28,
        offset_internal: 0x4C,
        property_link_next: 0x58,
    },
};

// Tested on Unreal Physics
/// Unreal Engine 5.3, x86-64.
pub const UNREAL_5_3_X86_64: Profile = Profile {
    pointer_size: PointerSize::Bit64,
    g_engine: G_ENGINE[0],
    g_world: G_WORLD[0],
    fname_pool: FNAME_POOL[0],
    object: ObjectOffsets {
        fname: 0x18,
        class: 0x10,
    },
    class: ClassOffsets {
        super_field: 0x40,
        property_link: 0x50,
    },
    property: PropertyOffsets {
        fname: 0x20,
        offset_internal: 0x44,
        property_link_next: 0x48,
    },
};
