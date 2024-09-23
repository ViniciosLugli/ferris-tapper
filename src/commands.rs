use crate::{cli::NetworkArgs, error::NetworkError, network_manager::NetworkManager};
use log::{info, warn};
use owo_colors::OwoColorize;

pub async fn start_network_configuration(
	network_manager: &NetworkManager,
	args: &NetworkArgs,
) -> Result<(), NetworkError> {
	let (interface_a, interface_b) = args.get_interfaces();

	println!("{}", format!("⚙️  Starting TAP network configuration").bold().green());
	info!("Starting TAP network configuration for {} and {}", interface_a, interface_b);

	let interface_color = interface_a.bold().cyan().to_string();
	let peer_color = interface_b.bold().blue().to_string();

	for interface in [&interface_a, &interface_b] {
		let peer_interface = if interface == &interface_a { &interface_b } else { &interface_a };
		let interface_display = if interface == &interface_a { &interface_color } else { &peer_color };

		println!("\n{}", format!("🔧 Configuring interface: {}", interface_display).bold().yellow());

		println!("  🚧 Removing old ingress qdisc configuration...");
		network_manager.remove_interface_configuration(interface).await?;

		println!("  🚿 Flushing IP addresses...");
		network_manager.flush_ip_addresses(interface).await?;

		println!("  🔧 Adding ingress qdisc...");
		match network_manager.add_qdisc_to_interface(interface).await {
			Ok(_) => info!("Added ingress qdisc to {}", interface),
			Err(NetworkError::RtnetlinkError(e)) if e.to_string().contains("File exists") => {
				println!("  ⚠️  Ingress qdisc already exists. Skipping...");
				warn!("Ingress qdisc already exists on {}. Skipping...", interface);
			}
			Err(e) => return Err(e),
		}

		let peer_display = if interface == &interface_a { &peer_color } else { &interface_color };
		println!("  📊 Adding traffic filter with peer interface: {}", peer_display);
		network_manager.add_traffic_filter(interface, peer_interface).await?;

		println!("  🚫 Disabling IPv6...");
		if let Err(e) = network_manager.disable_ipv6(interface) {
			println!("  ⚠️  Failed to disable IPv6: {}. Continuing...", e.to_string().yellow());
			warn!("Failed to disable IPv6 on {}: {}. Continuing...", interface, e);
		}

		println!("  🔍 Enabling promiscuous mode...");
		network_manager.set_promiscuous_mode(interface, true).await?;
	}

	println!("\n{}", "✅ Network configuration completed successfully!".green().bold());
	info!("Network configuration completed successfully!");
	Ok(())
}

pub async fn stop_network_configuration(
	network_manager: &NetworkManager,
	args: &NetworkArgs,
) -> Result<(), NetworkError> {
	let (interface_a, interface_b) = args.get_interfaces();

	println!("{}", format!("⚙️  Stopping TAP network configuration").bold().red());
	info!("Stopping TAP network configuration for {} and {}", interface_a, interface_b);

	let interface_color = interface_a.bold().cyan().to_string();
	let peer_color = interface_b.bold().blue().to_string();

	for interface in [&interface_a, &interface_b] {
		let interface_display = if interface == &interface_a { &interface_color } else { &peer_color };

		println!("\n{}", format!("🔧 Reverting configuration for interface: {}", interface_display).bold().yellow());

		println!("  🚧 Removing ingress qdisc configuration...");
		network_manager.remove_interface_configuration(interface).await?;

		println!("  ✅ Enabling IPv6...");
		if let Err(e) = network_manager.enable_ipv6(interface) {
			println!("  ⚠️  Failed to enable IPv6: {}. Continuing...", e.to_string().yellow());
			warn!("Failed to enable IPv6 on {}: {}. Continuing...", interface, e);
		}

		println!("  🔍 Disabling promiscuous mode...");
		if let Err(e) = network_manager.set_promiscuous_mode(interface, false).await {
			println!("  ⚠️  Failed to disable promiscuous mode: {}. Continuing...", e.to_string().yellow());
			warn!("Failed to disable promiscuous mode on {}: {}. Continuing...", interface, e);
		}
	}

	println!("\n{}", "✅ Network configuration stopped successfully!".green().bold());
	info!("Network configuration stopped successfully!");
	Ok(())
}

pub async fn show_network_status(network_manager: &NetworkManager, args: &NetworkArgs) -> Result<(), NetworkError> {
	let (interface_a, interface_b) = args.get_interfaces();
	for interface in [&interface_a, &interface_b] {
		let status = network_manager.get_interface_status(interface).await?;
		println!("{}", status);
	}
	Ok(())
}
