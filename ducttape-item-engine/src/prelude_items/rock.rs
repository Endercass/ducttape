use crate::{
    add_base_attribute, add_base_attributes,
    attribute::AttributeType,
    item::{DummyHook, EngineHook, Item, ItemMut, SpecialAbility, Stats},
};

use super::stats::BasicStats;

#[derive(Debug, Clone)]
pub struct Rock<THook: EngineHook = DummyHook> {
    stats: BasicStats,

    phantom: std::marker::PhantomData<THook>,
}

impl Default for Rock {
    fn default() -> Self {
        Self::new()
    }
}

impl Rock {
    pub fn new() -> Self {
        let stats = BasicStats::new();
        let mut rock = Rock {
            stats,
            phantom: std::marker::PhantomData,
        };

        rock.populate();

        rock
    }

    pub fn populate(&mut self) {
        add_base_attributes!(self,
            {
                AttributeType::Sharpness =>  2.0,
                AttributeType::Durability => 50.0,
                AttributeType::Weight =>      5.0,
                AttributeType::Strength =>   10.0,
                AttributeType::Agility =>     5.0,
                AttributeType::Reach =>       5.0
            }
        );
    }
}

impl<THook: EngineHook> Item<THook> for Rock<THook> {
    fn get_name(&self) -> String {
        "🪨".into()
    }

    fn get_ident(&self) -> String {
        "rock".into()
    }

    fn get_stats(&self) -> Box<dyn Stats> {
        Box::new(self.stats.clone())
    }

    fn special_abilities(&self) -> Vec<Box<dyn SpecialAbility<THook>>> {
        Vec::new()
    }

    fn get_texture(&self) -> Option<image::DynamicImage> {
        None
    }
}

impl<THook: EngineHook> ItemMut for Rock<THook> {
    fn get_stats_mut(&mut self) -> &mut dyn Stats {
        &mut self.stats
    }
}

#[cfg(test)]
mod tests {
    use valence_text::IntoText;

    use crate::{attribute::AttributeParser, text_renderer::ansi_renderer::AnsiRenderer};

    use super::*;

    #[test]
    fn rock_attrs() {
        let rock = Rock::new();

        let stats = rock.get_stats();
        let parser = AttributeParser::from(stats.get_all_attributes());

        let debug_txt = rock.get_name().into_text() + "\n---\n" + parser.clone();

        // Print out the item info for debugging
        println!("{}", debug_txt.to_ansi_string());
    }
}
