pub mod hud;
pub mod player;
pub mod singletons;

use bevy::{prelude::*, state::app::StatesPlugin};
use godot::prelude::*;

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
    app.add_plugins(StatesPlugin)
        .init_state::<GameState>()
        .add_systems(OnEnter(GameState::Playing), player::spawn);
}
