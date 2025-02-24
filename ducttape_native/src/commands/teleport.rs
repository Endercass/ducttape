use clap::Parser;

#[derive(Debug, Parser)]
pub struct TeleportCommand {
    /// The UUID of the entity to teleport, or nothing to teleport the player
    #[clap()]
    pub id: Option<String>,

    #[clap()]
    pub x: f32,

    #[clap()]
    pub y: f32,
}

// impl TeleportCommand {
//     pub fn execute(&self, state: &mut NativeGameState) {
//     }
// }
