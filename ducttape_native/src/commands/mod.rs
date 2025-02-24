pub mod teleport;

#[derive(Debug, clap::Subcommand)]
pub enum Command {
    Teleport(teleport::TeleportCommand),
}

#[derive(Debug, clap::Parser)]
#[command(multicall = true)]
pub struct Cli {
    #[command(subcommand)]
    command: Command,
}
