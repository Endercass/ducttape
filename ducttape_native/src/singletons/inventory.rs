use ducttape_item_engine::{
    item::{DummyHook, ItemCollection as _, ItemCollectionSized, ItemRegistry, ItemStack},
    prelude_items::{air::Air, dev_tablet::DevTablet, rock::Rock},
};
use godot::global::godot_print;
use lazy_static::lazy_static;
use maplit::hashmap;
use std::sync::{Arc, Mutex};

use crate::{item::rope::Rope, template::loader::ItemTemplate};

// static INVENTORY: Mutex<Option<ItemCollectionSized>> = Mutex::new(None);

lazy_static! {
    pub static ref INVENTORY: Mutex<ItemCollectionSized> = Mutex::new(generate_sample_inventory());
    pub static ref ITEM_REGISTRY: Mutex<ItemRegistry<DummyHook>> =
        Mutex::new(create_item_registry());
}

fn generate_sample_inventory() -> ItemCollectionSized {
    let mut registry = ITEM_REGISTRY.lock().unwrap();

    let mut inventory = ItemCollectionSized::new(16);
    inventory
        .add_item(ItemStack::new(registry.get("rock").unwrap().clone(), 3))
        .expect("Failed to add item to inventory");

    inventory
        .add_item(ItemStack::new(registry.get("rock").unwrap().clone(), 1))
        .expect("Failed to add item to inventory");

    inventory
        .add_item(ItemStack::new(registry.get("rock").unwrap().clone(), 2))
        .expect("Failed to add item to inventory");

    inventory
        .add_item(ItemStack::new(registry.get("rope").unwrap().clone(), 1))
        .expect("Failed to add item to inventory");

    inventory
        .add_item(ItemStack::new(registry.get("dev_tablet").unwrap().clone(), 1))
        .expect("Failed to add item to inventory");

    let spear_template = ItemTemplate::load_template("spear").unwrap();

    let components = hashmap! {
        "shaft".to_owned() => registry.get("rock").unwrap().clone(),
        "tip".to_owned() => registry.get("rock").unwrap().clone(),
    };

    let spear = spear_template.populate_template(components);

    registry.register("spear".to_owned(), Arc::new(spear));

    inventory
        .add_item(ItemStack::new(registry.get("spear").unwrap().clone(), 1))
        .expect("Failed to add item to inventory");

    inventory
}

pub fn create_item_registry() -> ItemRegistry<DummyHook> {
    let mut registry: ItemRegistry = ItemRegistry::new();

    registry.register("rock".to_owned(), {
        let rock = Rock::new();
        godot_print!("Registered rock item: {:?}", rock);
        Arc::new(rock)
    });

    registry.register("air".to_owned(), {
        let air = Air::new();
        godot_print!("Registered air item: {:?}", air);
        Arc::new(air)
    });

    registry.register("rope".to_owned(), {
        let rope = Rope::new();
        godot_print!("Registered rope item: {:?}", rope);
        Arc::new(rope)
    });

     registry.register("dev_tablet".to_owned(), {
        let dev_tablet = DevTablet::new();
        godot_print!("Registered dev_tablet item: {:?}", dev_tablet);
        Arc::new(dev_tablet)
    });

    registry
}
