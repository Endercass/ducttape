pub mod game_entities;
pub mod hud;
pub mod player;
pub mod remote;
pub mod singletons;

use std::future::pending;

use bevy::{prelude::*, state::app::StatesPlugin, tasks::futures_lite};
use bevy_godot4::prelude::{AsPhysicsSystem, ErasedGd};
use godot::prelude::*;
use remote::dbus_remote_provider::{DbusCommandQueue, DbusRemoteProvider};

#[derive(Debug, Default, Clone, Eq, PartialEq, Hash, States)]
pub enum GameState {
    #[default]
    Playing,
}

struct DuctTapeNative;

#[gdextension]
unsafe impl ExtensionLibrary for DuctTapeNative {
    fn on_level_init(level: godot::prelude::InitLevel) {
        if level == godot::prelude::InitLevel::Core {
            godot::private::class_macros::registry::class::auto_register_classes(level);
            let mut app_builder_func = bevy_godot4::APP_BUILDER_FN.lock().unwrap();
            if app_builder_func.is_none() {
                *app_builder_func = Some(Box::new(build_app));
            }
        }
    }
}

fn build_app(app: &mut App) {
    let queue = DbusCommandQueue::default();

    app.add_plugins(StatesPlugin)
        .insert_resource(queue.clone())
        .init_state::<GameState>()
        .add_systems(OnEnter(GameState::Playing), game_entities::player::spawn)
        .add_systems(
            Update,
            remote::dbus_remote_provider::remote_provider_system.as_physics_system(),
        )
        .add_systems(Update, transform_system.as_physics_system());

    godot_print!("Starting Remote provider (main)");

    // smol::spawn(async move {
    //     godot_print!("Starting Remote provider (smol)");

    //     let provider: DbusRemoteProvider = queue.clone().into(); // Clone the Arc to move it into the thread

    //     godot_print!("Provider created");

    //     let conn = zbus::connection::Builder::session()
    //         .unwrap()
    //         .name("me.endercass.ducttape.Remote")
    //         .unwrap()
    //         .serve_at("/me/endercass/ducttape/Remote", provider)
    //         .unwrap()
    //         .build()
    //         .await;

    //     godot_print!("Remote provider started: {:?}", conn);
    // })
    // .detach();

    godot_print!("Starting Remote provider (smol)");

    // let provider: DbusRemoteProvider = queue.clone().into(); // Clone the Arc to move it into the thread

    // let conn = zbus::blocking::connection::Builder::session()
    //     .unwrap()
    //     .name("me.endercass.ducttape.Remote")
    //     .unwrap()
    //     .serve_at("/me/endercass/ducttape/Remote", provider)
    //     .unwrap()
    //     .build()
    //     .unwrap();
    // godot_print!("Remote provider started: {:?}", conn);

    // Box::leak(Box::new(conn));

    // let task = smol::spawn();

    // fork

    std::thread::spawn(move || {
        futures_lite::future::block_on(async move {
            let provider: DbusRemoteProvider = queue.clone().into(); // Clone the Arc to move it into the thread

            let conn = zbus::connection::Builder::session()
                .unwrap()
                .name("me.endercass.ducttape.Remote")
                .unwrap()
                .serve_at("/me/endercass/ducttape/Remote", provider)
                .unwrap()
                .build()
                .await;

            match conn {
                Ok(conn) => {
                    println!("Remote provider started: {:?}", conn);
                    pending::<()>().await;
                }
                Err(e) => println!("Error starting Remote provider: {:?}", e),
            }
        });
    });
}

// #[derive(Resource)]
// struct EntityManipulationSystems(HashMap<String, SystemId>);

// This just straight up doesn't work, but it doesn't matter right now.
fn transform_system(mut query: Query<(&mut ErasedGd, &Transform)>) {
    for (mut node, transform) in query.iter_mut() {
        let mut node = node.get::<Node2D>();
        node.set_position(Vector2::new(
            transform.translation.x,
            transform.translation.y,
        ));
        node.set_rotation(transform.rotation.to_axis_angle().1);
        node.set_scale(Vector2::new(transform.scale.x, transform.scale.y));
    }
}
