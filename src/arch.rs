pub struct Architecture
{
    pub reader_cnt: usize,
    pub reader_wait_mask: usize,
}

#[cfg(all(unix, target_pointer_width = "32"))]
pub const ARCH: Architecture = Architecture {
    reader_cnt: 15,
    reader_wait_mask: 0b010101010101010101010101010101,
};

#[cfg(all(unix, target_pointer_width = "64"))]
pub const ARCH: Architecture = Architecture {
    reader_cnt: 30,
    reader_wait_mask: 0b010101010101010101010101010101010101010101010101010101010101,
};
