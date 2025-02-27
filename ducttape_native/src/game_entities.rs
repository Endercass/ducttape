use bevy::{
    ecs::{entity::Entity, system::Commands},
    math::Vec3,
    transform::components::Transform,
};
use bevy_godot4::prelude::{ErasedGdResource, GodotScene};
use godot::{classes::Resource, prelude::*, tools::load};

macro_rules! spawnable {
    ($name:ident) => {
        paste::paste! {
            pub mod [<$name:snake>] {
                use super::*;

                pub fn spawn(mut commands: Commands) {
                    super::GameEntity::$name.spawn(&mut commands);
                }
            }
        }
    };
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, zbus::zvariant::Type,
)]
pub enum GameEntity {
    Player,
}

spawnable!(Player);

impl GameEntity {
    fn get_scene(&self) -> Gd<Resource> {
        match self {
            GameEntity::Player => load::<Resource>("res://Player.tscn"),
        }
    }

    pub fn spawn(&self, commands: &mut Commands) -> Entity {
        let entity = commands.spawn(GodotScene::from_resource(ErasedGdResource::new(
            self.get_scene(),
        )));

        entity.id()
    }

    pub fn spawn_with_position(&self, commands: &mut Commands, position: Vector2) -> Entity {
        let mut entity = commands.spawn(GodotScene::from_resource(ErasedGdResource::new(
            self.get_scene(),
        )));

        entity
            .commands()
            .spawn(Transform::from_translation(Vec3::new(
                position.x, position.y, 0.0,
            )));

        entity.id()
        // .insert_resource(Transform::from_translation(Vec3::new(
        //     position.0, position.1, 0.0,
        // )));
        // .insert_bundle((Transform::from_translation(Vec3::new(
        //     position.0, position.1, 0.0,
        // )),));
    }
}
