use crate::{signature::Signature, PointerSize};

/// How a global is found: code that uses the global, with the displacement
/// of the global masked out. The signature always holds 24 bytes, so a
/// shorter one is written out with wildcards after it.
#[derive(Copy, Clone, Debug)]
pub struct Anchor {
    /// The code, with the displacement masked out.
    pub signature: Signature<24>,
    /// Where the displacement starts inside a match.
    pub displacement: u8,
    /// Where the instruction after the one with the displacement starts
    /// inside a match. The displacement counts from there.
    pub next_instruction: u8,
}

/// The members of `UObject` the module reads.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ObjectOffsets {
    /// The `FName` of the object.
    pub fname: u8,
    /// The pointer to the class of the object.
    pub class: u8,
}

/// The members of `UClass` the module reads.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ClassOffsets {
    /// The pointer to the parent class.
    pub super_field: u8,
    /// The pointer to the first property.
    pub property_link: u8,
}

/// The members of a property the module reads. Properties are `UProperty`
/// objects before Unreal Engine 4.25 and `FProperty` fields from 4.25 on.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct PropertyOffsets {
    /// The `FName` of the property.
    pub fname: u8,
    /// Where the value of the property sits inside an instance of its class.
    pub offset_internal: u8,
    /// The pointer to the next property.
    pub property_link_next: u8,
}

/// A complete layout of the engine objects the module reads, with the
/// anchors that find `GEngine`, `GWorld` and the `FNamePool`.
///
/// Profiles are deliberately exhaustive. If a future ASR release needs more
/// layout information, custom profiles must provide it rather than silently
/// inheriting an offset that may be wrong. Built-in profiles are available
/// in [`profiles`](super::profiles).
///
/// A custom profile lists the complete layout. Every field is written out
/// intentionally: using struct-update syntax would opt the custom profile in
/// to silently inheriting fields added to the source profile in the future.
///
/// ```
/// use asr::{
///     game_engine::unreal::{Anchor, ClassOffsets, ObjectOffsets, Profile, PropertyOffsets},
///     signature::Signature,
///     PointerSize,
/// };
///
/// const CUSTOM_PROFILE: Profile = Profile {
///     pointer_size: PointerSize::Bit64,
///     g_engine: Anchor {
///         signature: Signature::new(
///             "A8 01 75 ?? 48 C7 05 ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? ??",
///         ),
///         displacement: 7,
///         next_instruction: 15,
///     },
///     g_world: Anchor {
///         signature: Signature::new(
///             "80 7C 24 ?? 00 ?? ?? 48 8B 3D ?? ?? ?? ?? 48 ?? ?? ?? ?? ?? ?? ?? ?? ??",
///         ),
///         displacement: 10,
///         next_instruction: 14,
///     },
///     fname_pool: Anchor {
///         signature: Signature::new(
///             "74 09 48 8D 15 ?? ?? ?? ?? EB 16 ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? ?? ??",
///         ),
///         displacement: 5,
///         next_instruction: 9,
///     },
///     object: ObjectOffsets {
///         fname: 0x18,
///         class: 0x10,
///     },
///     class: ClassOffsets {
///         super_field: 0x40,
///         property_link: 0x50,
///     },
///     property: PropertyOffsets {
///         fname: 0x28,
///         offset_internal: 0x4C,
///         property_link_next: 0x58,
///     },
/// };
/// ```
#[derive(Copy, Clone, Debug)]
pub struct Profile {
    /// The pointer width this profile was measured for.
    pub pointer_size: PointerSize,
    /// The anchor that finds `GEngine`.
    pub g_engine: Anchor,
    /// The anchor that finds `GWorld`.
    pub g_world: Anchor,
    /// The anchor that finds the `FNamePool`.
    pub fname_pool: Anchor,
    /// Offsets within `UObject`.
    pub object: ObjectOffsets,
    /// Offsets within `UClass`.
    pub class: ClassOffsets,
    /// Offsets within a property.
    pub property: PropertyOffsets,
}
