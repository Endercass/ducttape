use std::collections::HashMap;


use godot::{prelude::{GodotConvert, Var}};
use valence_text::Text;

use crate::attribute::{Attribute, AttributeType};

/// The dynamic Item trait that represents an abstract game item. This trait will provide methods for getting every item's stats, name, and description.
pub trait Item<THook = DummyHook> {
    /// Get the item's name.
    fn get_name(&self) -> Text;
    /// Get the item's stats. (This will be the final stats after component attributes, if present, and from the item's base stats)
    fn get_stats(&self) -> Box<dyn Stats>;
    /// Get the item's stats mutable.
    fn get_stats_mut(&mut self) -> &mut dyn Stats;
    /// Get the item's special abilities. (This will be the final special abilities after component attributes, if present, and from the item's base special abilities)
    fn get_special_abilities(&self) -> Vec<Box<dyn SpecialAbility<THook>>>;
    /// Color of the item (This will eventually be a more robust texture system but for now no rendering will happen in the item engine)
    fn get_color(&self) -> u32;
}

pub trait SpecialAbility<THook> {
    fn get_name(&self) -> Text;
    fn get_hooks(&self) -> Vec<THook>;
}

type DummyHook = ();


/// The Stats trait that represents an abstract game item's stats.
pub trait Stats {
    fn get_attribute(&self, at: AttributeType, id: uuid::Uuid) -> Attribute;
    fn get_attributes(&self, at: AttributeType) -> Vec<Attribute>;

    fn get_all_attributes(&self) -> HashMap<AttributeType, Vec<Attribute>>;

    fn push_attribute(&mut self, at: AttributeType, attribute: Attribute);
    fn push_attributes(&mut self, attributes: HashMap<AttributeType, Attribute>);

    fn set_attribute(&mut self, at: AttributeType, id: uuid::Uuid, attribute: Attribute);
    fn set_attributes(&mut self, at: AttributeType, attributes: Vec<Attribute>);

    fn remove_attribute(&mut self, at: AttributeType, id: uuid::Uuid);
    fn remove_attributes(&mut self, at: AttributeType);
}

pub struct ItemCollection {
    items: Vec<Box<dyn Item>>,
}

impl ItemCollection {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
        }
    }

    pub fn add_item(&mut self, item: Box<dyn Item>) {
        self.items.push(item);
    }

    pub fn get_item(&self, index: usize) -> Option<&Box<dyn Item>> {
        self.items.get(index)
    }

    pub fn get_item_mut(&mut self, index: usize) -> Option<&mut Box<dyn Item>> {
        self.items.get_mut(index)
    }

    pub fn remove_item(&mut self, index: usize) -> Box<dyn Item> {
        self.items.remove(index)
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn iter(&self) -> std::slice::Iter<Box<dyn Item>> {
        self.items.iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<Box<dyn Item>> {
        self.items.iter_mut()
    }

    pub fn into_iter(self) -> std::vec::IntoIter<Box<dyn Item>> {
        self.items.into_iter()
    }

    pub fn clear(&mut self) {
        self.items.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn get_items(&self) -> &Vec<Box<dyn Item>> {
        &self.items
    }

    pub fn get_items_mut(&mut self) -> &mut Vec<Box<dyn Item>> {
        &mut self.items
    }
}

pub mod macros {
    #[macro_export]
    /// Take a struct implementing Item and a f32 to construct a base attribute
    macro_rules! base_attribute {
        ($item:expr, $value:expr) => {
            $crate::attribute::Attribute {
                uuid: uuid::Uuid::new_v4(),
                reason: $crate::attribute::AttributeReason::Display { name: $item.get_name() },
                priority: 0,
                modifier: $crate::attribute::AttributeModifier::Set($value),
            }
        };
    }

    #[macro_export]
    /// Add a base attribute to an item
    macro_rules! add_base_attribute {
        ($item:expr, $at:expr, $value:expr) => {
            let name = $item.get_name();
            
            $item.get_stats_mut().push_attribute($at, 
                $crate::attribute::Attribute {
                    uuid: uuid::Uuid::new_v4(),
                    reason: $crate::attribute::AttributeReason::Display { name },
                    priority: 0,
                    modifier: $crate::attribute::AttributeModifier::Set($value),
                }
            );
        };
    }

    #[macro_export]
    /// Add a set of base attributes to an item
    macro_rules! add_base_attributes {
        ($item:expr, { $($at:expr => $value:expr),* }) => {
            $(
                add_base_attribute!($item, $at, $value);
            )*
        };
    }
}