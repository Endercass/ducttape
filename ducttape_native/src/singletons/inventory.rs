use ducttape_item_engine::{
    item::{ItemCollection as _, ItemCollectionSized},
    prelude_items::rock::Rock,
};
use lazy_static::lazy_static;
use std::sync::Mutex;

// static INVENTORY: Mutex<Option<ItemCollectionSized>> = Mutex::new(None);

lazy_static! {
    pub static ref INVENTORY: Mutex<ItemCollectionSized> = Mutex::new(generate_sample_inventory());
}

fn generate_sample_inventory() -> ItemCollectionSized {
    let mut inventory = ItemCollectionSized::new(16);
    inventory
        .add_item(Box::new(Rock::new()))
        .expect("Failed to add item to inventory");
    inventory
}
