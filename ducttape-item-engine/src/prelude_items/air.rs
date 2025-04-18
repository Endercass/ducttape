use crate::item::{DummyHook, EngineHook, Item, ItemMut, ItemStack, ItemTexture, SpecialAbility, Stats};
use std::sync::Arc;

use super::stats::BasicStats;

#[derive(Debug, Clone)]
pub struct Air<THook: EngineHook = DummyHook> {
    stats: BasicStats,
    phantom: std::marker::PhantomData<THook>,
}

impl<THook: EngineHook> Default for Air<THook> {
    fn default() -> Self {
        Self::new()
    }
}

impl<THook: EngineHook> Air<THook> {
    pub fn new() -> Self {
        Self {
            stats: BasicStats::new(),
            phantom: std::marker::PhantomData,
        }
    }

    pub fn new_itemstack() -> ItemStack<THook> {
        ItemStack::new(Arc::new(Self::new()), 1)
    }
}

impl<THook: EngineHook> Item<THook> for Air<THook> {
    fn get_name(&self) -> String {
        "☁️".into()
    }

    fn get_ident(&self) -> String {
        "air".into()
    }

    fn get_stats(&self) -> Box<dyn Stats> {
        Box::new(self.stats.clone())
    }

    fn special_abilities(&self) -> Vec<Box<dyn SpecialAbility<THook>>> {
        vec![]
    }

    fn get_texture(&self) -> ItemTexture {
        // None
        super::AIR_TEXTURE.clone()
    }
}

impl<THook: EngineHook> ItemMut for Air<THook> {
    fn get_stats_mut(&mut self) -> &mut dyn Stats {
        &mut self.stats
    }
}
