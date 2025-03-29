use crate::item::{DummyHook, EngineHook, Item, ItemMut, ItemTexture, SpecialAbility, Stats};

use super::{stats::BasicStats, DEV_TABLET_TEXTURE};

#[derive(Debug, Clone)]
pub struct DevTablet<THook: EngineHook = DummyHook> {
    phantom: std::marker::PhantomData<THook>,

    stats: BasicStats,
}

impl Default for DevTablet {
    fn default() -> Self {
        Self::new()
    }
}

impl DevTablet {
    pub fn new() -> Self {
        
        DevTablet {
            phantom: std::marker::PhantomData,
            stats: BasicStats::new(),
        }
    }
}

impl<THook: EngineHook> Item<THook> for DevTablet<THook> {
    fn get_name(&self) -> String {
        "Tablet".into()
    }

    fn get_ident(&self) -> String {
        "dev_tablet".into()
    }

    fn get_stats(&self) -> Box<dyn Stats> {
        Box::new(self.stats.clone())
    }

    fn special_abilities(&self) -> Vec<Box<dyn SpecialAbility<THook>>> {
        Vec::new()
    }

    fn get_texture(&self) -> ItemTexture {
        DEV_TABLET_TEXTURE.clone()
    }
}

impl<THook: EngineHook> ItemMut for DevTablet<THook> {
    fn get_stats_mut(&mut self) -> &mut dyn Stats {
        &mut self.stats
    }
}