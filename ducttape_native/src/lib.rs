pub mod commands;
pub mod hud;
pub mod player;
pub mod singletons;

use bevy::{prelude::*, state::app::StatesPlugin};
use godot::prelude::*;
use uuid::Uuid;
use valence_text::{color::NamedColor, IntoText, Text};

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum WellKnownEntity {
    Player,
}

impl WellKnownEntity {
    pub fn get_id(&self) -> Uuid {
        match self {
            WellKnownEntity::Player => "f702c33e-fc43-4ec5-9922-aba77be47e70",
        }
        .parse()
        .unwrap()
    }

    pub fn get_name(&self) -> String {
        match self {
            WellKnownEntity::Player => "player".to_string(),
        }
    }

    pub fn get_friendly_name(&self) -> Text {
        match self {
            WellKnownEntity::Player => "Player".color(NamedColor::Green),
        }
    }

    pub fn well_known_entities() -> Vec<Self> {
        vec![WellKnownEntity::Player]
    }
}

#[derive(Default, States, Debug, Hash, PartialEq, Eq, Clone)]
pub struct GameState {
    well_known_entities: Vec<WellKnownEntity>,
}

impl GameState {
    pub fn new() -> Self {
        let mut well_known_entities = WellKnownEntity::well_known_entities();
        well_known_entities.sort_by_key(|e| e.get_name());
        GameState {
            well_known_entities,
        }
    }

    pub fn get_well_known_entities(&self) -> &Vec<WellKnownEntity> {
        &self.well_known_entities
    }

    pub fn get_well_known_entity(&self, id: Uuid) -> Option<&WellKnownEntity> {
        self.well_known_entities.iter().find(|e| e.get_id() == id)
    }

    pub fn get_well_known_entity_by_name(&self, name: &str) -> Option<&WellKnownEntity> {
        self.well_known_entities
            .iter()
            .find(|e| e.get_name() == name)
    }
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
    app.add_plugins(StatesPlugin).init_state::<GameState>();
}

// struct DuctTapeNative;
