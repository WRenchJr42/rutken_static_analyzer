use crate::binary::BinaryReader;
use crate::dex::header::DexHeader;
use crate::errors::ApkError;
use crate::dex::string_id::StringIds;

#[derive(Debug)]
pub struct DexDocument {
    pub header: DexHeader,
    pub string_ids: StringIds,
}

pub struct DexParser;

impl DexParser {
    pub fn parse(bytes: &[u8]) -> Result<DexDocument, ApkError> {
        println!("DEX bytes: {}", bytes.len());
        let mut reader = BinaryReader::new(bytes);
        println!("Reading header...");
        let header = DexHeader::parse(&mut reader)?;
        println!("Header OK!");
        println!("{:#?}", header);
        println!(
        "string_ids_size={}, string_ids_off={}",
        header.string_ids_size,
        header.string_ids_off
        );
        let string_ids = StringIds::parse(&mut reader, header.string_ids_size, header.string_ids_off)?;
        println!("Read {} string IDs", string_ids.strings.len());
        Ok(DexDocument{
            header,
            string_ids,
        })
    }
}
