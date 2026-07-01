use crate::axml::attribute::Attribute;
use crate::axml::string_pool::StringPool;

#[derive(Debug, Clone)]
pub struct ResolvedAttribute {
    pub namespace: Option<String>,
    pub name: String,
    pub value: String,
}

pub fn resolve_attribute(attribute: &Attribute, pool: &StringPool) -> ResolvedAttribute {
    let namespace = if attribute.namespace != u32::MAX {
        Some(pool.strings[attribute.namespace as usize].clone())
    } else {
        None
    };
    let name = pool.strings[attribute.name as usize].clone();
    let value = if attribute.raw_value != u32::MAX {
        pool.strings[attribute.raw_value as usize].clone()
    } else {
        attribute.data.to_string()
    };
    
    ResolvedAttribute {
        namespace,
        name,
        value,
    }
}


