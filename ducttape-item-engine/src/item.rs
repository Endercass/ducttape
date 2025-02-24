use std::{
    any::{Any, TypeId},
    collections::HashMap,
    fmt::Debug,
    pin::Pin,
    sync::{Arc, Mutex, MutexGuard},
};

use godot::{classes::ip::Type, meta::ToGodot, prelude::GodotConvert};
use valence_text::Text;

use crate::{
    attribute::{Attribute, AttributeType},
    prelude_items::air::{Air, IsAir},
};

/// The dynamic Item trait that represents an abstract game item. This trait will provide methods for getting every item's stats, name, and description.
pub trait Item<THook = DummyHook>: Any + Debug + Send + Sync {
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

pub struct ItemCollectionUnsized {
    items: Vec<Box<dyn Item>>,
}

pub struct ItemCollectionSized {
    items: Vec<Box<dyn Item>>,
    size: usize,
}

impl ItemCollectionSized {
    pub fn new(size: usize) -> Self {
        let mut items: Vec<Box<dyn Item>> = Vec::with_capacity(size);

        for _ in 0..size {
            items.push(Box::new(Air::new()));
        }

        Self { items, size }
    }
}

#[derive(Debug)]
pub enum ItemCollectionError {
    /// The item collection is full and cannot accept any more items.
    Full,
    /// The item collection is empty and cannot remove any more items.
    Empty,
    /// The item collection does not contain the item.
    NotFound,
}

impl std::fmt::Display for ItemCollectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ItemCollectionError::Full => write!(f, "Item collection is full"),
            ItemCollectionError::Empty => write!(f, "Item collection is empty"),
            ItemCollectionError::NotFound => write!(f, "Item not found in collection"),
        }
    }
}

impl std::error::Error for ItemCollectionError {}

pub type ItemCollectionResult<T> = Result<T, ItemCollectionError>;

pub trait ItemCollection {
    fn add_item(&mut self, item: Box<dyn Item>) -> ItemCollectionResult<()>;
    fn get_item(&self, index: usize) -> ItemCollectionResult<&Box<dyn Item>>;
    fn get_item_mut(&mut self, index: usize) -> ItemCollectionResult<&mut Box<dyn Item>>;
    fn remove_item(&mut self, index: usize) -> ItemCollectionResult<Box<dyn Item>>;
    fn len(&self) -> usize;
    fn iter(&self) -> std::slice::Iter<Box<dyn Item>>;
    fn iter_mut(&mut self) -> std::slice::IterMut<Box<dyn Item>>;
    fn into_iter(self) -> std::vec::IntoIter<Box<dyn Item>>;
    fn clear(&mut self);
    fn is_empty(&self) -> bool;
    fn get_items(&self) -> &Vec<Box<dyn Item>>;
    fn get_items_mut(&mut self) -> &mut Vec<Box<dyn Item>>;
}

impl ItemCollection for ItemCollectionUnsized {
    fn add_item(&mut self, item: Box<dyn Item>) -> ItemCollectionResult<()> {
        self.items.push(item);
        Ok(())
    }

    fn get_item(&self, index: usize) -> ItemCollectionResult<&Box<dyn Item>> {
        if let Some(item) = self.items.get(index) {
            if item.is_air() {
                println!("Found air item: {:?}", item.get_name());
                Err(ItemCollectionError::NotFound)
            } else {
                Ok(item)
            }
        } else {
            Err(ItemCollectionError::NotFound)
        }
    }

    fn get_item_mut(&mut self, index: usize) -> ItemCollectionResult<&mut Box<dyn Item>> {
        if let Some(item) = self.items.get_mut(index) {
            if item.is_air() {
                println!("Found air item: {:?}", item.get_name());
                Err(ItemCollectionError::NotFound)
            } else {
                Ok(item)
            }
        } else {
            Err(ItemCollectionError::NotFound)
        }
    }

    fn remove_item(&mut self, index: usize) -> ItemCollectionResult<Box<dyn Item>> {
        // Replace the item with air, or returning the item if it's not air
        if let Some(item) = self.items.get_mut(index) {
            if item.is_air() {
                println!("Found air item: {:?}", item.get_name());
                Err(ItemCollectionError::NotFound)
            } else {
                Ok(std::mem::replace(item, Box::new(Air::new())))
            }
        } else {
            Err(ItemCollectionError::NotFound)
        }
    }

    fn len(&self) -> usize {
        self.items.len()
    }

    fn iter(&self) -> std::slice::Iter<Box<dyn Item>> {
        self.items.iter()
    }

    fn iter_mut(&mut self) -> std::slice::IterMut<Box<dyn Item>> {
        self.items.iter_mut()
    }

    fn into_iter(self) -> std::vec::IntoIter<Box<dyn Item>> {
        self.items.into_iter()
    }

    fn clear(&mut self) {
        self.items.clear();
    }

    fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    fn get_items(&self) -> &Vec<Box<dyn Item>> {
        &self.items
    }

    fn get_items_mut(&mut self) -> &mut Vec<Box<dyn Item>> {
        &mut self.items
    }
}

impl ItemCollection for ItemCollectionSized {
    fn add_item(&mut self, item: Box<dyn Item>) -> ItemCollectionResult<()> {
        self.items.iter_mut().find(|i| i.is_air()).map_or_else(
            || Err(ItemCollectionError::Full),
            |i| {
                *i = item;
                Ok(())
            },
        )
    }

    fn get_item(&self, index: usize) -> ItemCollectionResult<&Box<dyn Item>> {
        if let Some(item) = self.items.get(index) {
            if item.is_air() {
                println!("Found air item: {:?}", item.get_name());
                Err(ItemCollectionError::NotFound)
            } else {
                Ok(item)
            }
        } else {
            Err(ItemCollectionError::NotFound)
        }
    }

    fn get_item_mut(&mut self, index: usize) -> ItemCollectionResult<&mut Box<dyn Item>> {
        if let Some(item) = self.items.get_mut(index) {
            if item.is_air() {
                println!("Found air item: {:?}", item.get_name());
                Err(ItemCollectionError::NotFound)
            } else {
                Ok(item)
            }
        } else {
            Err(ItemCollectionError::NotFound)
        }
    }

    fn remove_item(&mut self, index: usize) -> ItemCollectionResult<Box<dyn Item>> {
        // Replace the item with air, or returning the item if it's not air
        if let Some(item) = self.items.get_mut(index) {
            if item.is_air() {
                println!("Found air item: {:?}", item.get_name());
                Err(ItemCollectionError::NotFound)
            } else {
                Ok(std::mem::replace(item, Box::new(Air::new())))
            }
        } else {
            Err(ItemCollectionError::NotFound)
        }
    }

    fn len(&self) -> usize {
        self.items.len()
    }

    fn iter(&self) -> std::slice::Iter<Box<dyn Item>> {
        self.items.iter()
    }

    fn iter_mut(&mut self) -> std::slice::IterMut<Box<dyn Item>> {
        self.items.iter_mut()
    }

    fn into_iter(self) -> std::vec::IntoIter<Box<dyn Item>> {
        self.items.into_iter()
    }

    fn clear(&mut self) {
        self.items.clear();
    }

    fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    fn get_items(&self) -> &Vec<Box<dyn Item>> {
        &self.items
    }

    /// Get the items mutable reference.
    fn get_items_mut(&mut self) -> &mut Vec<Box<dyn Item>> {
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
                reason: $crate::attribute::AttributeReason::Display {
                    name: $item.get_name(),
                },
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

            $item.get_stats_mut().push_attribute(
                $at,
                $crate::attribute::Attribute {
                    uuid: uuid::Uuid::new_v4(),
                    reason: $crate::attribute::AttributeReason::Display { name },
                    priority: 0,
                    modifier: $crate::attribute::AttributeModifier::Set($value),
                },
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
