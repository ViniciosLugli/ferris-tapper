use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "ferris-tapper")]
#[command(about = "A network tap tool written in Rust", long_about = None)]
pub struct Cli {
	#[command(subcommand)]
	command: Commands,
}

impl Cli {
	pub fn get_command(&self) -> &Commands {
		&self.command
	}
}

#[derive(Debug, Args)]
pub struct NetworkArgs {
	interface_a: String,
	interface_b: String,
}

impl NetworkArgs {
	pub fn get_interfaces(&self) -> (&str, &str) {
		(&self.interface_a, &self.interface_b)
	}
}

#[derive(Debug, Subcommand)]
pub enum Commands {
	Start(NetworkArgs),
	Stop(NetworkArgs),
	Status(NetworkArgs),
}
