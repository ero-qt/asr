pub(super) struct IL2CPPOffsets {
    pub(super) assembly: AssemblyOffsets,
    pub(super) image: ImageOffsets,
    pub(super) class: ClassOffsets,
    pub(super) field: FieldInfoOffsets,
}

pub(super) struct AssemblyOffsets {
    pub(super) image: u8,
    pub(super) aname: Option<u8>, // Either this or ImageOffsets::assembly_name locates the name
}

pub(super) struct ImageOffsets {
    pub(super) assembly_name: Option<u8>, // Either this or AssemblyOffsets::aname locates the name
    pub(super) type_count: u8,
    pub(super) type_start: TypeStart,
}

/// Where an image keeps the index of its first type in the type table.
#[derive(Copy, Clone)]
pub(super) enum TypeStart {
    /// The index sits in the image, at this offset.
    Inline(u8),
    /// A pointer sits in the image at this offset. The index sits where it
    /// points.
    Handle(u8),
}

pub(super) struct ClassOffsets {
    pub(super) name: u8,
    pub(super) namespace: u8,
    pub(super) parent: u8,
    pub(super) fields: u8,
    pub(super) static_fields: u8,
    pub(super) field_count: u16,
}

pub(super) struct FieldInfoOffsets {
    pub(super) name: u8,
    pub(super) offset: u8,
    pub(super) struct_size: u8,
}
