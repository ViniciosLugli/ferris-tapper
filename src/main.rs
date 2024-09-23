mod cli;
mod commands;
mod error;
mod network_manager;
mod types;

use clap::Parser;
use cli::{Cli, Commands};
use commands::{show_network_status, start_network_configuration, stop_network_configuration};
use env_logger;
use network_manager::NetworkManager;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	env_logger::init();

	let args = Cli::parse();
	let network_manager = NetworkManager::new().await?;

	match args.get_command() {
		Commands::Start(network_args) => {
			start_network_configuration(&network_manager, &network_args).await?;
		}
		Commands::Stop(network_args) => {
			stop_network_configuration(&network_manager, &network_args).await?;
		}
		Commands::Status(network_args) => {
			show_network_status(&network_manager, &network_args).await?;
		}
	}

	Ok(())
}
