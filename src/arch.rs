pub struct Architecture
{
    pub reader_cnt: usize,
    pub reader_lock_mask: usize,
}

#[cfg(all(unix, target_pointer_width = "2"))]
pub const ARCH: Architecture = Architecture {
    reader_cnt: 1,
    reader_lock_mask: 0b1,
};

#[cfg(all(unix, target_pointer_width = "32"))]
pub const ARCH: Architecture = Architecture {
    reader_cnt: 31,
    reader_lock_mask: 0b1111111111111111111111111111111,
};

#[cfg(all(unix, target_pointer_width = "64"))]
pub const ARCH: Architecture = Architecture {
    reader_cnt: 63,
    reader_lock_mask: 0b111111111111111111111111111111111111111111111111111111111111111,
};
