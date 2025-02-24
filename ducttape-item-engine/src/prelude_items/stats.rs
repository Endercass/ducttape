use std::collections::HashMap;

use crate::{
    attribute::{Attribute, AttributeType},
    item::Stats,
};

#[derive(Debug, Clone)]
pub struct BasicStats {
    attributes: HashMap<AttributeType, Vec<Attribute>>,
}

impl Stats for BasicStats {
    fn get_attribute(&self, at: AttributeType, id: uuid::Uuid) -> Attribute {
        self.attributes
            .get(&at)
            .unwrap()
            .iter()
            .find(|a| a.uuid == id)
            .unwrap()
            .clone()
    }

    fn get_attributes(&self, at: AttributeType) -> Vec<Attribute> {
        self.attributes.get(&at).unwrap().clone()
    }

    fn get_all_attributes(&self) -> HashMap<AttributeType, Vec<Attribute>> {
        self.attributes.clone()
    }

    fn push_attribute(&mut self, at: AttributeType, attribute: Attribute) {
        self.attributes.entry(at).or_default().push(attribute);
    }

    fn push_attributes(&mut self, attributes: HashMap<AttributeType, Attribute>) {
        for (at, attribute) in attributes {
            self.push_attribute(at, attribute);
        }
    }

    fn set_attribute(&mut self, at: AttributeType, id: uuid::Uuid, attribute: Attribute) {
        let vec = self.attributes.entry(at).or_default();

        if let Some(index) = vec.iter().position(|a| a.uuid == id) {
            vec[index] = attribute;
        }
    }

    fn set_attributes(&mut self, at: AttributeType, attributes: Vec<Attribute>) {
        self.attributes.insert(at, attributes);
    }

    fn remove_attribute(&mut self, at: AttributeType, id: uuid::Uuid) {
        if let Some(vec) = self.attributes.get_mut(&at) {
            vec.retain(|a| a.uuid != id);
        }
    }

    fn remove_attributes(&mut self, at: AttributeType) {
        self.attributes.remove(&at);
    }
}

impl BasicStats {
    pub fn new() -> Self {
        BasicStats {
            attributes: HashMap::new(),
        }
    }
}

impl From<HashMap<AttributeType, Vec<Attribute>>> for BasicStats {
    fn from(attributes: HashMap<AttributeType, Vec<Attribute>>) -> Self {
        BasicStats { attributes }
    }
}

impl Default for BasicStats {
    fn default() -> Self {
        BasicStats::new()
    }
}

pub struct BasicStatsBuilder {
    attributes: HashMap<AttributeType, Vec<Attribute>>,
}

impl Default for BasicStatsBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl BasicStatsBuilder {
    pub fn new() -> Self {
        BasicStatsBuilder {
            attributes: HashMap::new(),
        }
    }

    pub fn with_attribute(mut self, at: AttributeType, attribute: Attribute) -> Self {
        self.attributes.entry(at).or_default().push(attribute);
        self
    }

    pub fn with_attribute_vecs(
        mut self,
        attributes: HashMap<AttributeType, Vec<Attribute>>,
    ) -> Self {
        for (at, attribute) in attributes {
            self.attributes.entry(at).or_default().extend(attribute);
        }
        self
    }

    pub fn with_attribute_vec(mut self, at: AttributeType, attributes: Vec<Attribute>) -> Self {
        self.attributes.entry(at).or_default().extend(attributes);
        self
    }

    pub fn build(self) -> BasicStats {
        BasicStats {
            attributes: self.attributes,
        }
    }
}
