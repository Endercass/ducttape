use valence_text::Text;

use crate::{
    add_base_attribute, add_base_attributes,
    attribute::AttributeType,
    item::{Item, SpecialAbility, Stats},
};

use super::stats::BasicStats;

#[derive(Debug)]
pub struct Rock {
    stats: BasicStats,
}

impl Default for Rock {
    fn default() -> Self {
        Self::new()
    }
}

impl Rock {
    pub fn new() -> Self {
        let stats = BasicStats::new();
        let mut rock = Rock { stats };

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

#[cfg(test)]
mod tests {
    use crate::{attribute::AttributeParser, text_renderer::ansi_renderer::AnsiRenderer};

    use super::*;

    #[test]
    fn rock_attrs() {
        let rock = Rock::new();

        let stats = rock.get_stats();
        let parser = AttributeParser::from(stats.get_all_attributes());

        let debug_txt = rock.get_name() + "\n---\n" + parser.clone();

        // Print out the item info for debugging
        println!("{}", debug_txt.to_ansi_string());
    }
}
