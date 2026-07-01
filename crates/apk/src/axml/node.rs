use crate::axml::resolve::ResolvedAttribute;

#[derive(Debug, Clone)]
pub struct XmlNode {
    pub name: String,
    pub attributes: Vec<ResolvedAttribute>,
    pub children: Vec<XmlNode>,
}

impl XmlNode {
    pub fn new(name: String, attributes: Vec<ResolvedAttribute>) -> Self{
        Self {
            name,
            attributes,
            children: Vec::new(),
        }
    }
}
