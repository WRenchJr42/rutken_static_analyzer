use crate::axml::attribute::Attribute;
use crate::axml::string_pool::StringPool;

#[derive(Debug, Clone)]
pub struct ResolvedAttribute {
    pub namespace: Option<String>,
    pub name: String,
    pub value: String,
}

/// Fetch a string from the pool by index, falling back to a sentinel value
/// when the index is out of range (untrusted/malformed AXML input).
fn resolve_string(pool: &StringPool, idx: u32) -> String {
    pool.strings
        .get(idx as usize)
        .cloned()
        .unwrap_or_else(|| format!("<bad_string:{}>", idx))
}

pub fn resolve_attribute(attribute: &Attribute, pool: &StringPool) -> ResolvedAttribute {
    let namespace = if attribute.namespace != u32::MAX {
        Some(resolve_string(pool, attribute.namespace))
    } else {
        None
    };
    let name = resolve_string(pool, attribute.name);
    let value = if attribute.raw_value != u32::MAX {
        resolve_string(pool, attribute.raw_value)
    } else {
        attribute.data.to_string()
    };
    
    ResolvedAttribute {
        namespace,
        name,
        value,
    }
}


