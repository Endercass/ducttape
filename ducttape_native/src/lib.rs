pub mod player;

use godot::prelude::*;

struct DuctTapeNative;

#[gdextension]
unsafe impl ExtensionLibrary for DuctTapeNative {}