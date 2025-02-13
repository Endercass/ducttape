use std::collections::HashMap;

use valence_text::{IntoText, Text};

use crate::{item::{Item, SpecialAbility, Stats}, attribute::{Attribute, AttributeModifier, AttributeReason, AttributeType}};

pub struct Rock;

impl Rock {
    pub fn new() -> Self {
        Rock
    }
}

#[cfg(test)]
mod tests {
    use crate::{attribute::AttributeParser, text_renderer::ansi_renderer::AnsiRenderer};

    use super::*;

    #[test]
    fn rock_attrs() {
        let rock = Rock::new();
        let stats = rock.get_stats();
        let parser = AttributeParser::from(stats.get_all_attributes());

        let debug_txt = 
            rock.get_name()
            + "\n---\n" 
            + parser.clone();

        // Print out the item info for debugging
        println!("{}", debug_txt.to_ansi_string());
    }
}

impl Item for Rock {
    fn get_name(&self) -> Text {
        "🪨".into()
    }

    fn get_stats(&self) -> Box<dyn Stats> {
        Box::new(RockStats::default())
    }

    fn get_special_abilities(&self) -> Vec<Box<dyn SpecialAbility<()>>> {
        Vec::new()
    }

    fn get_color(&self) -> u32 {
        0x808080 // Gray color
    }
}

#[derive(Clone)]
pub struct RockStats {
    attributes: HashMap<AttributeType, Vec<Attribute>>,
}

impl Stats for RockStats {
    fn get_attribute(&self, at: AttributeType, id: uuid::Uuid) -> Attribute {
        self.attributes.get(&at).unwrap().iter().find(|a| a.uuid == id).unwrap().clone()
    }

    fn get_attributes(&self, at: AttributeType) -> Vec<Attribute> {
        self.attributes.get(&at).unwrap().clone()
    }

    fn get_all_attributes(&self) -> HashMap<AttributeType, Vec<Attribute>> {
        self.attributes.clone()
    }

    fn push_attribute(&mut self, at: AttributeType, attribute: Attribute) {
        self.attributes.entry(at).or_insert_with(Vec::new).push(attribute);
    }

    fn push_attributes(&mut self, attributes: HashMap<AttributeType, Attribute>) {
        for (at, attribute) in attributes {
            self.push_attribute(at, attribute);
        }
    }

    fn set_attribute(&mut self, at: AttributeType, id: uuid::Uuid, attribute: Attribute) {
        let vec = self.attributes.entry(at).or_insert_with(Vec::new);

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
impl Default for RockStats {
    fn default() -> Self {
        let mut attributes = HashMap::new();
        attributes.insert(AttributeType::Durability, vec![
            Attribute {
                uuid: uuid::Uuid::new_v4(),
                reason: AttributeReason::Display { 
                    name: "🪨".into_text(), 
                },
                priority: 0,
                modifier: AttributeModifier::Set(50.0),
            }
        ]);
        attributes.insert(AttributeType::Weight, vec![
            Attribute {
                uuid: uuid::Uuid::new_v4(),
                reason: AttributeReason::Display { 
                    name: "🪨".into_text(), 
                },
                priority: 0,
                modifier: AttributeModifier::Set(5.0),
            }
        ]);
        attributes.insert(AttributeType::Strength, vec![
            Attribute {
                uuid: uuid::Uuid::new_v4(),
                reason: AttributeReason::Display { 
                    name: "🪨".into_text(), 
                },
                priority: 0,
                modifier: AttributeModifier::Set(10.0),
            }
        ]);

        RockStats {
            attributes,
        }
    }
}