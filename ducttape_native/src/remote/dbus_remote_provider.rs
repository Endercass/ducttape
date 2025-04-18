use std::sync::{Arc, Mutex};

use bevy::ecs::prelude::*;
use ducttape_item_engine::item::{ItemCollection as _, ItemStack};
use godot::prelude::*;
use zbus::interface;

use crate::{
    game_entities::GameEntity,
    singletons::inventory::{INVENTORY, ITEM_REGISTRY},
};

pub fn remote_provider_system(mut commands: Commands, queue: Res<DbusCommandQueue>) {
    let mut queue = queue.0.lock().unwrap();
    for command in queue.drain(..) {
        match command {
            DbusCommand::SpawnEntity(entity, position) => {
                entity.spawn_with_position(&mut commands, position);
            }
        }
    }
}

// Define possible commands that can be queued
#[derive(Debug)]
enum DbusCommand {
    SpawnEntity(GameEntity, Vector2),
}

/// Shared queue to store DBus commands
#[derive(Resource, Clone, Default)]
pub struct DbusCommandQueue(Arc<Mutex<Vec<DbusCommand>>>);

pub struct DbusRemoteProvider {
    command_queue: DbusCommandQueue,
}

impl From<DbusCommandQueue> for DbusRemoteProvider {
    fn from(command_queue: DbusCommandQueue) -> Self {
        Self { command_queue }
    }
}

#[interface(name = "me.endercass.ducttape.RemoteProvider")]
impl DbusRemoteProvider {
    fn spawn_entity(&self, entity: GameEntity, position: (f32, f32)) {
        let mut queue = self.command_queue.0.lock().unwrap();
        queue.push(DbusCommand::SpawnEntity(
            entity,
            Vector2::new(position.0, position.1),
        ));
    }

    fn add_item(&self, item: String, amount: u32) {
        println!("Adding item: {}", item);

        let registry = ITEM_REGISTRY.lock().unwrap();
        let mut inventory = INVENTORY.lock().unwrap();

        if let Some(item) = registry.get(&item) {
            println!("Found item: {:?}", item);

            inventory
                .add_item(ItemStack::new(item.clone(), amount))
                .unwrap();
        } else {
            println!("Item not found: {}", item);
        }
    }

    // fn remove_entity(&self, entity: Entity) -> Result<(), String>;
    // fn set_position(&self, entity: Entity, position: Vec2) -> Result<(), String>;
    // fn set_velocity(&self, entity: Entity, velocity: Vec2) -> Result<(), String>;
    // fn set_rotation(&self, entity: Entity, rotation: f32) -> Result<(), String>;
    // fn set_scale(&self, entity: Entity, scale: Vec2) -> Result<(), String>;
    // fn add_item(&self, item: Item) -> Result<(), String>;
    // fn remove_item(&self, item: Item) -> Result<(), String>;
}
