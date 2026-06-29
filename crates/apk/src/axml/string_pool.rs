#[derive(Debug)]
pub struct StringPoolHeader {
    pub chunk_type: u16,
    pub header_size: u16,
    pub chunk_size: u32,
    pub string_count: u32,
    pub style_count: u32,
    pub flags: u32,
    pub strings_start: u32,
    pub styles_start: u32,
}


