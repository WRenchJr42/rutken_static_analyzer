// `DexParser`/`DexDocument` (parser.rs) and their pub fields intentionally
// stay `pub` because they are the crate's supported entry points into DEX
// parsing (see dex::model and external consumers). The id-table modules
// below are implementation details of `DexDocument` and are not referenced
// outside this crate, so they are demoted to `pub(crate)`.
pub(crate) mod header;
pub mod parser;
pub(crate) mod string_id;
pub(crate) mod strings;
pub(crate) mod type_id;
pub(crate) mod proto_id;
pub(crate) mod method_id;
pub(crate) mod field_id;
pub(crate) mod class_def;
pub(crate) mod class_data;
pub(crate) mod code_item;
pub(crate) mod disasm;
pub mod instruction;
pub(crate) mod opcode;
pub mod model;
