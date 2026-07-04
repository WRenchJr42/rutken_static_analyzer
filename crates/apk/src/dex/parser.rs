use crate::binary::BinaryReader;
use crate::dex::header::DexHeader;
use crate::errors::ApkError;
use crate::dex::string_id::StringIds;
use crate::dex::strings::DexStrings;
use crate::dex::type_id::TypeIds;
use crate::dex::proto_id::ProtoIds;

#[derive(Debug)]
pub struct DexDocument {
    pub header: DexHeader,
    pub string_ids: StringIds,
    pub strings: DexStrings,
    pub type_ids: TypeIds,
    pub proto_ids: ProtoIds,
}

pub struct DexParser;

impl DexParser {
    pub fn parse(bytes: &[u8]) -> Result<DexDocument, ApkError> {
        println!("DEX bytes: {}", bytes.len());
        let mut reader = BinaryReader::new(bytes);
        let header = DexHeader::parse(&mut reader)?;
        let type_ids = TypeIds::parse(&mut reader, header.type_ids_size, header.type_ids_off)?;
        let proto_ids = ProtoIds::parse(&mut reader, header.proto_ids_size, header.proto_ids_off)?;
        println!("{:#?}", header);
        println!("string_ids_size={}, string_ids_off={}", header.string_ids_size, header.string_ids_off);
        let string_ids = StringIds::parse(&mut reader, header.string_ids_size, header.string_ids_off)?;
        let strings = DexStrings::parse(&mut reader, &string_ids)?;
        println!("Read {} string IDs", string_ids.strings.len());
        Ok(DexDocument{
            header,
            string_ids,
            strings,
            type_ids,
            proto_ids,
        })
    }
}
