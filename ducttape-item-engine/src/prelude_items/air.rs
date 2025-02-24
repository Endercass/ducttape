use std::collections::HashMap;

use valence_text::{IntoText, Text};

use crate::{
    attribute::{Attribute, AttributeType},
    item::{Item, SpecialAbility, Stats},
};

use super::stats::BasicStats;

#[derive(Debug, Default)]
pub struct Air {
    stats: BasicStats,
}

impl Air {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Item for Air {
    fn get_name(&self) -> Text {
        "☁️".into_text()
    }

    fn get_stats(&self) -> Box<dyn Stats> {
        Box::new(self.stats.clone())
    }

    fn get_stats_mut(&mut self) -> &mut dyn Stats {
        &mut self.stats
    }

    fn get_special_abilities(&self) -> Vec<Box<dyn SpecialAbility<()>>> {
        vec![]
    }

    fn get_color(&self) -> u32 {
        0x000000
    }
}

pub trait IsAir {
    fn is_air(&self) -> bool;
}

impl IsAir for dyn Item {
    fn is_air(&self) -> bool {
        self.type_id() == std::any::TypeId::of::<Air>()
    }
}
