use std::collections::HashMap;


use valence_text::Text;

use crate::attribute::{Attribute, AttributeType};

/// The dynamic Item trait that represents an abstract game item. This trait will provide methods for getting every item's stats, name, and description.
pub trait Item<THook = DummyHook> {
    /// Get the item's name.
    fn get_name(&self) -> Text;
    /// Get the item's stats. (This will be the final stats after component attributes, if present, and from the item's base stats)
    fn get_stats(&self) -> Box<dyn Stats>;
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