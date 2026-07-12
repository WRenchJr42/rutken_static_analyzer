// `AxmlParser`/`AxmlDocument` (parser.rs) are the crate's public entry point
// for parsing binary AndroidManifest.xml. The chunk/table modules below are
// implementation details of that parser and are not referenced outside this
// crate, so they are demoted to `pub(crate)`.
pub(crate) mod header;
pub mod parser;
pub(crate) mod string_pool;
pub(crate) mod chunks;
pub(crate) mod resource_map;
pub(crate) mod constants;
pub(crate) mod namespace;
pub(crate) mod element;
pub(crate) mod attribute;
pub(crate) mod resolve;
pub(crate) mod end_element;
pub(crate) mod end_namespace;
pub(crate) mod node;
