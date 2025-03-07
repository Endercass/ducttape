use ducttape_item_engine::{
    add_base_attribute, add_base_attributes,
    attribute::AttributeType,
    prelude_items::stats::BasicStats,
    item::{DummyHook, EngineHook, Item, ItemMut, SpecialAbility, Stats},
};

#[derive(Debug, Clone)]
pub struct Rope<THook: EngineHook = DummyHook> {
    stats: BasicStats,
    phantom: std::marker::PhantomData<THook>,
}

impl Default for Rope {
    fn default() -> Self {
        Self::new()
    }
}

impl Rope {
    pub fn new() -> Self {
        let stats = BasicStats::new();
        let mut rock = Rope {
            stats,
            phantom: std::marker::PhantomData,
        };

        rock.populate();

        rock
    }

    pub fn populate(&mut self) {
        add_base_attributes!(self,
            {
                AttributeType::Sharpness =>  0.0,
                AttributeType::Durability => 150.0,
                AttributeType::Weight =>      5.0,
                AttributeType::Strength =>   10.0,
                AttributeType::Agility =>     5.0,
                AttributeType::Reach =>       10.0
            }
        );
    }
}

impl<THook: EngineHook> Item<THook> for Rope<THook> {
    fn get_name(&self) -> String {
        "🪢".into()
    }

    fn get_ident(&self) -> String {
        "rope".into()
    }

    fn get_stats(&self) -> Box<dyn Stats> {
        Box::new(self.stats.clone())
    }

    fn special_abilities(&self) -> Vec<Box<dyn SpecialAbility<THook>>> {
        Vec::new()
    }

    fn get_texture(&self) -> Option<image::DynamicImage> {
        Some(super::ROPE_TEXTURE.clone())
    }
}

impl<THook: EngineHook> ItemMut for Rope<THook> {
    fn get_stats_mut(&mut self) -> &mut dyn Stats {
        &mut self.stats
    }
}

