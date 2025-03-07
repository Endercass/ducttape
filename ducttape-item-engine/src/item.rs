use std::{any::Any, collections::HashMap, fmt::Debug, sync::Arc};

use dyn_clone::DynClone;
use image::DynamicImage;

use crate::{
    attribute::{Attribute, AttributeType},
    prelude_items::air::Air,
};

/// The dynamic Item trait that represents an abstract game item. This trait will provide methods for getting every item's stats, name, and description.
pub trait Item<THook: EngineHook = DummyHook>: Any + Debug + Send + Sync + DynClone {
    /// Get the item's name.
    fn get_name(&self) -> String;
    /// Get the item's identifier.
    fn get_ident(&self) -> String;
    /// Get the item's stats. (This will be the final stats after component attributes, if present, and from the item's base stats)
    fn get_stats(&self) -> Box<dyn Stats>;
    /// Get the item's special abilities. (This will be the final special abilities after component attributes, if present, and from the item's base special abilities)
    fn special_abilities(&self) -> Vec<Box<dyn SpecialAbility<THook>>>;
    /// Get the item's texture
    fn get_texture(&self) -> Option<DynamicImage>;
}

dyn_clone::clone_trait_object!(<THook> Item<THook>);

/// The ItemMut trait that represents an abstract game item that can be mutated.
pub trait ItemMut {
    /// Get the item's stats mutable.
    fn get_stats_mut(&mut self) -> &mut dyn Stats;
}

pub trait SpecialAbility<THook: EngineHook = DummyHook>:
    Debug + Send + Sync + DynClone + 'static
{
    fn get_name(&self) -> String;
    fn get_hooks(&self) -> Vec<THook>;
}

dyn_clone::clone_trait_object!(<THook> SpecialAbility<THook>);

pub trait EngineHook: Debug + Send + Sync + Clone + 'static {}

#[derive(Debug, Clone)]
pub struct DummyHook {}

impl EngineHook for DummyHook {}

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

pub struct ItemCollectionUnsized<THook: EngineHook = DummyHook> {
    items: Vec<ItemStack<THook>>,
    listeners: Vec<Box<dyn Fn(ItemCollectionEvent<THook>) + Send + Sync>>,
}

pub struct ItemCollectionSized<THook: EngineHook = DummyHook> {
    items: Vec<ItemStack<THook>>,
    listeners: Vec<Box<dyn Fn(ItemCollectionEvent<THook>) + Send + Sync>>,
    size: usize,
}

impl<THook: EngineHook> ItemCollectionSized<THook> {
    pub fn new(size: usize) -> Self {
        let mut items: Vec<ItemStack<THook>> = Vec::with_capacity(size);

        for _ in 0..size {
            items.push(Air::new_itemstack());
        }

        Self {
            items,
            size,
            listeners: Vec::new(),
        }
    }

    pub fn get_size(&self) -> usize {
        self.size
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

#[derive(Debug)]
pub enum ItemCollectionEvent<THook: EngineHook = DummyHook> {
    Add {
        index: usize,
        item: Arc<ItemStack<THook>>,
    },
    Remove {
        index: usize,
        item: Arc<ItemStack<THook>>,
    },
    Clear,
    ManualRefresh,
}

pub trait ItemCollection<THook: EngineHook = DummyHook> {
    fn add_item(&mut self, item: ItemStack<THook>) -> ItemCollectionResult<()>;
    fn get_item(&self, index: usize) -> ItemCollectionResult<&ItemStack<THook>>;
    fn get_item_mut(&mut self, index: usize) -> ItemCollectionResult<&mut ItemStack<THook>>;
    fn remove_item(&mut self, index: usize) -> ItemCollectionResult<ItemStack<THook>>;
    fn len(&self) -> usize;
    fn iter(&self) -> std::slice::Iter<ItemStack<THook>>;
    fn iter_mut(&mut self) -> std::slice::IterMut<ItemStack<THook>>;
    fn refresh(&mut self); // Call this when you want to refresh the collection, usually after a manual change with get_iter_mut or get_item_mut
    fn into_iter(self) -> std::vec::IntoIter<ItemStack<THook>>;
    fn clear(&mut self);
    fn is_empty(&self) -> bool;
    fn get_items(&self) -> &Vec<ItemStack<THook>>;
    fn get_items_mut(&mut self) -> &mut Vec<ItemStack<THook>>;
    fn listen(&mut self, f: Box<dyn Fn(ItemCollectionEvent<THook>) + Send + Sync>);
}

impl<THook: EngineHook> ItemCollection<THook> for ItemCollectionUnsized<THook> {
    fn add_item(&mut self, item: ItemStack<THook>) -> ItemCollectionResult<()> {
        self.listeners.iter().for_each(|f| {
            f(ItemCollectionEvent::Add {
                index: self.items.len() - 1,
                item: Arc::new(item.clone()),
            })
        });
        self.items.push(item);
        Ok(())
    }

    fn get_item(&self, index: usize) -> ItemCollectionResult<&ItemStack<THook>> {
        if let Some(item) = self.items.get(index) {
            if item.get_ident() == "air" {
                println!("Found air item: {:?}", item.get_name());
                Err(ItemCollectionError::NotFound)
            } else {
                Ok(item)
            }
        } else {
            Err(ItemCollectionError::NotFound)
        }
    }

    fn get_item_mut(&mut self, index: usize) -> ItemCollectionResult<&mut ItemStack<THook>> {
        if let Some(item) = self.items.get_mut(index) {
            if item.get_ident() == "air" {
                println!("Found air item: {:?}", item.get_name());
                Err(ItemCollectionError::NotFound)
            } else {
                Ok(item)
            }
        } else {
            Err(ItemCollectionError::NotFound)
        }
    }

    fn remove_item(&mut self, index: usize) -> ItemCollectionResult<ItemStack<THook>> {
        // Replace the item with air, or returning the item if it's not air
        if let Some(item) = self.items.get_mut(index) {
            if item.get_ident() == "air" {
                println!("Found air item: {:?}", item.get_name());
                Err(ItemCollectionError::NotFound)
            } else {
                let item = std::mem::replace(item, Air::new_itemstack());
                self.listeners.iter().for_each(|f| {
                    f(ItemCollectionEvent::Remove {
                        index,
                        item: Arc::new(item.clone()),
                    })
                });
                Ok(item)
            }
        } else {
            Err(ItemCollectionError::NotFound)
        }
    }

    fn len(&self) -> usize {
        self.items.len()
    }

    fn iter(&self) -> std::slice::Iter<ItemStack<THook>> {
        self.items.iter()
    }

    fn iter_mut(&mut self) -> std::slice::IterMut<ItemStack<THook>> {
        self.items.iter_mut()
    }

    fn into_iter(self) -> std::vec::IntoIter<ItemStack<THook>> {
        self.items.into_iter()
    }

    fn refresh(&mut self) {
        self.listeners
            .iter()
            .for_each(|f| f(ItemCollectionEvent::ManualRefresh));
    }

    fn clear(&mut self) {
        self.items.clear();
    }

    fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    fn get_items(&self) -> &Vec<ItemStack<THook>> {
        &self.items
    }

    fn get_items_mut(&mut self) -> &mut Vec<ItemStack<THook>> {
        &mut self.items
    }

    fn listen(&mut self, f: Box<dyn Fn(ItemCollectionEvent<THook>) + Send + Sync>) {
        self.listeners.push(f);
    }
}

impl<THook: EngineHook> ItemCollection<THook> for ItemCollectionSized<THook> {
    fn add_item(&mut self, item: ItemStack<THook>) -> ItemCollectionResult<()> {
        if let Some(index) = self.items.iter().position(|i| i.get_ident() == "air") {
            self.listeners.iter().for_each(|f| {
                f(ItemCollectionEvent::Add {
                    index,
                    item: Arc::new(item.clone()),
                })
            });
            self.items[index] = item;
            Ok(())
        } else {
            Err(ItemCollectionError::Full)
        }
    }

    fn get_item(&self, index: usize) -> ItemCollectionResult<&ItemStack<THook>> {
        if let Some(item) = self.items.get(index) {
            if item.get_ident() == "air" {
                println!("Found air item: {:?}", item.get_name());
                Err(ItemCollectionError::NotFound)
            } else {
                Ok(item)
            }
        } else {
            Err(ItemCollectionError::NotFound)
        }
    }

    fn get_item_mut(&mut self, index: usize) -> ItemCollectionResult<&mut ItemStack<THook>> {
        if let Some(item) = self.items.get_mut(index) {
            if item.get_ident() == "air" {
                println!("Found air item: {:?}", item.get_name());
                Err(ItemCollectionError::NotFound)
            } else {
                Ok(item)
            }
        } else {
            Err(ItemCollectionError::NotFound)
        }
    }

    fn remove_item(&mut self, index: usize) -> ItemCollectionResult<ItemStack<THook>> {
        // Replace the item with air, or returning the item if it's not air
        if let Some(item) = self.items.get_mut(index) {
            if item.get_ident() == "air" {
                println!("Found air item: {:?}", item.get_name());
                Err(ItemCollectionError::NotFound)
            } else {
                let item = std::mem::replace(item, Air::new_itemstack());
                self.listeners.iter().for_each(|f| {
                    f(ItemCollectionEvent::Remove {
                        index,
                        item: Arc::new(item.clone()),
                    })
                });
                Ok(item)
            }
        } else {
            Err(ItemCollectionError::NotFound)
        }
    }

    fn len(&self) -> usize {
        self.items.len()
    }

    fn iter(&self) -> std::slice::Iter<ItemStack<THook>> {
        self.items.iter()
    }

    fn iter_mut(&mut self) -> std::slice::IterMut<ItemStack<THook>> {
        self.items.iter_mut()
    }

    fn into_iter(self) -> std::vec::IntoIter<ItemStack<THook>> {
        self.items.into_iter()
    }

    fn refresh(&mut self) {
        self.listeners
            .iter()
            .for_each(|f| f(ItemCollectionEvent::ManualRefresh));
    }

    fn clear(&mut self) {
        self.items.clear();
    }

    fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    fn get_items(&self) -> &Vec<ItemStack<THook>> {
        &self.items
    }

    /// Get the items mutable reference.
    fn get_items_mut(&mut self) -> &mut Vec<ItemStack<THook>> {
        &mut self.items
    }

    fn listen(&mut self, f: Box<dyn Fn(ItemCollectionEvent<THook>) + Send + Sync>) {
        self.listeners.push(f);
    }
}

pub struct ItemRegistry<THook: EngineHook = DummyHook> {
    items: HashMap<String, Arc<dyn Item<THook>>>,
}

impl<THook: EngineHook> Default for ItemRegistry<THook> {
    fn default() -> Self {
        Self::new()
    }
}

impl<THook: EngineHook> ItemRegistry<THook> {
    pub fn new() -> Self {
        Self {
            items: HashMap::new(),
        }
    }

    pub fn register(&mut self, ident: String, item: Arc<dyn Item<THook>>) {
        self.items.insert(ident, item);
    }

    pub fn get(&self, ident: &str) -> Option<&Arc<dyn Item<THook>>> {
        self.items.get(ident)
    }

    pub fn get_mut(&mut self, ident: &str) -> Option<&mut Arc<dyn Item<THook>>> {
        self.items.get_mut(ident)
    }

    pub fn remove(&mut self, ident: &str) -> Option<Arc<dyn Item<THook>>> {
        self.items.remove(ident)
    }

    pub fn contains(&self, ident: &str) -> bool {
        self.items.contains_key(ident)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &Arc<dyn Item<THook>>)> {
        self.items.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&String, &mut Arc<dyn Item<THook>>)> {
        self.items.iter_mut()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn clear(&mut self) {
        self.items.clear();
    }

    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.items.keys()
    }

    pub fn values(&self) -> impl Iterator<Item = &Arc<dyn Item<THook>>> {
        self.items.values()
    }

    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut Arc<dyn Item<THook>>> {
        self.items.values_mut()
    }
}

impl IntoIterator for ItemRegistry {
    type Item = (String, Arc<dyn Item>);
    type IntoIter = std::collections::hash_map::IntoIter<String, Arc<dyn Item>>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

#[derive(Debug, Clone)]
pub struct ItemStack<THook: EngineHook = DummyHook> {
    item: Arc<dyn Item<THook>>,
    count: u32,
}

impl<THook: EngineHook> ItemStack<THook> {
    pub fn new(item: Arc<dyn Item<THook>>, count: u32) -> Self {
        Self { item, count }
    }

    pub fn get_item(&self) -> Arc<dyn Item<THook>> {
        self.item.clone()
    }

    pub fn get_count(&self) -> u32 {
        self.count
    }

    pub fn set_count(&mut self, count: u32) {
        self.count = count;
    }

    pub fn increment_count(&mut self, count: u32) {
        self.count += count;
    }

    pub fn decrement_count(&mut self, count: u32) {
        self.count -= count;
    }
}

impl<THook: EngineHook> Item<THook> for ItemStack<THook> {
    fn get_name(&self) -> String {
        self.item.get_name()
    }

    fn get_ident(&self) -> String {
        self.item.get_ident()
    }

    fn get_stats(&self) -> Box<dyn Stats> {
        self.item.get_stats()
    }

    fn special_abilities(&self) -> Vec<Box<dyn SpecialAbility<THook>>> {
        self.item.special_abilities()
    }

    fn get_texture(&self) -> Option<DynamicImage> {
        self.item.get_texture()
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
                    reason: $crate::attribute::AttributeReason::Display(name),
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
