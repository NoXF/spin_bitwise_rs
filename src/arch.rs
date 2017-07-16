pub struct Architecture
{
    pub reader_cnt: usize,
    pub reader_lease_mask: usize,
}

#[cfg(all(unix, target_pointer_width = "1"))]
pub const ARCH: Architecture = Architecture {
    reader_cnt: 1,
    reader_lease_mask: 0b01,
};

#[cfg(all(unix, target_pointer_width = "32"))]
pub const ARCH: Architecture = Architecture {
    reader_cnt: 15,
    reader_lease_mask: 0b010101010101010101010101010101,
};

#[cfg(all(unix, target_pointer_width = "64"))]
pub const ARCH: Architecture = Architecture {
    reader_cnt: 30,
    reader_lease_mask: 0b010101010101010101010101010101010101010101010101010101010101,
};
