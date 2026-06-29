#[derive(Debug)]
pub struct AxmlHeader{
    pub chunk_type: u16,
    pub header_size: u16,
    pub file_size: u32,
}
