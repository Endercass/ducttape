use std::collections::HashMap;

use valence_text::Text;

use crate::{add_base_attribute, add_base_attributes, attribute::{Attribute, AttributeType}, item::{Item, SpecialAbility, Stats}};

pub struct Rock {
    stats: RockStats,
}

impl Default for Rock {
    fn default() -> Self {
        Self::new()
    }
}

impl Rock {
    pub fn new() -> Self {
        let stats = RockStats::new();
        let mut rock = Rock {
            stats,
        };

        rock.populate();

        rock
    }

    pub fn populate(&mut self) {
        add_base_attributes!(self,
            {
                AttributeType::Durability => 50.0,
                AttributeType::Weight =>      5.0,
                AttributeType::Strength =>   10.0,
                AttributeType::Agility =>     5.0,
                AttributeType::Speed =>       5.0
            }
        );
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
        Box::new(self.stats.clone())
    }

    fn get_stats_mut(&mut self) -> &mut dyn Stats {
        &mut self.stats
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
impl RockStats {
    fn new() -> Self {
        RockStats {
            attributes: HashMap::new(),
        }
    }
}