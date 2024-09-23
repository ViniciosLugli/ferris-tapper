use futures_util::TryStreamExt;
use log::{info, warn};
use netlink_packet_route::link::LinkFlags;
use netlink_packet_route::link::LinkMessage;
use netlink_packet_route::tc::TcAttribute;
use netlink_packet_route::tc::TcMessage;
use rtnetlink::{new_connection, Handle};
use sysctl::Sysctl;

use crate::error::NetworkError;
use crate::types::InterfaceStatus;
use crate::types::QdiscInfo;
use crate::types::QdiscStats;

pub struct NetworkManager {
	handle: Handle,
}

impl NetworkManager {
	pub async fn new() -> Result<Self, NetworkError> {
		let (connection, handle, _) = new_connection()?;
		tokio::spawn(connection);
		Ok(Self { handle })
	}

	pub async fn add_traffic_filter(&self, interface: &str, peer_interface: &str) -> Result<(), NetworkError> {
		info!("Adding traffic filter to mirror traffic from {} to {}", interface, peer_interface);

		let index = self.get_interface_index(interface).await?;
		let peer_index = self.get_interface_index(peer_interface).await?;

		self.handle
			.traffic_filter(index)
			.add()
			.ingress()
			.protocol(0x0003)
			.redirect(peer_index.try_into().expect("Failed to convert peer index to u32"))
			.unwrap()
			.execute()
			.await?;

		info!("Added filter to mirror traffic from {} to {}", interface, peer_interface);
		Ok(())
	}

	pub async fn remove_interface_configuration(&self, interface: &str) -> Result<(), NetworkError> {
		info!("Removing configuration for {}", interface);

		let index = match self.get_interface_index(interface).await {
			Ok(index) => index,
			Err(NetworkError::NotFound(_)) => {
				info!("Interface {} not found. Skipping removal.", interface);
				return Ok(());
			}
			Err(e) => return Err(e),
		};

		match self.handle.qdisc().del(index).ingress().execute().await {
			Ok(_) => info!("Removed ingress qdisc from {}", interface),
			Err(e) if e.to_string().contains("No such file or directory") => {
				info!("No ingress qdisc found on {}. Skipping removal.", interface);
			}
			Err(e) => {
				warn!("Failed to remove ingress qdisc from {}: {}. Continuing...", interface, e);
			}
		}

		Ok(())
	}

	pub async fn add_qdisc_to_interface(&self, interface: &str) -> Result<(), NetworkError> {
		let index = self.get_interface_index(interface).await?;

		self.handle.qdisc().add(index).ingress().execute().await?;
		info!("Added ingress qdisc to {}", interface);

		Ok(())
	}

	pub async fn flush_ip_addresses(&self, interface: &str) -> Result<(), NetworkError> {
		let index = self.get_interface_index(interface).await?;

		let mut addresses = self
			.handle
			.address()
			.get()
			.set_link_index_filter(index.try_into().expect("Failed to convert index to u32"))
			.execute();

		while let Ok(Some(address_msg)) = addresses.try_next().await {
			self.handle.address().del(address_msg.clone()).execute().await?;
			info!("Flushed IP address: {:?}", address_msg);
		}

		Ok(())
	}

	pub async fn get_interface_index(&self, interface: &str) -> Result<i32, NetworkError> {
		let mut links = self.handle.link().get().match_name(interface.to_string()).execute();

		while let Ok(Some(link)) = links.try_next().await {
			return Ok(link.header.index.try_into().expect("Failed to convert link index to i32"));
		}

		Err(NetworkError::NotFound(format!("Could not find interface: {}", interface)))
	}

	pub fn disable_ipv6(&self, interface: &str) -> Result<(), NetworkError> {
		self.set_ipv6(interface, false)
	}

	pub fn enable_ipv6(&self, interface: &str) -> Result<(), NetworkError> {
		self.set_ipv6(interface, true)
	}

	pub fn set_ipv6(&self, interface: &str, enable: bool) -> Result<(), NetworkError> {
		let ctl_name = format!("net.ipv6.conf.{}.disable_ipv6", interface);
		let ctl = sysctl::Ctl::new(&ctl_name)
			.map_err(|_| NetworkError::SysctlError(format!("Could not get sysctl '{}'", ctl_name)))?;

		let value = if enable { "0" } else { "1" };
		let current_value = ctl
			.value_string()
			.map_err(|_| NetworkError::SysctlError(format!("Could not get value for sysctl '{}'", ctl_name)))?;

		if current_value != value {
			ctl.set_value_string(value).map_err(|_| {
				NetworkError::SysctlError(format!(
					"Failed to {} IPv6 on {}",
					if enable { "enable" } else { "disable" },
					interface
				))
			})?;
			info!("{} IPv6 for {}", if enable { "Enabled" } else { "Disabled" }, interface);
		} else {
			warn!("IPv6 already {} for {}", if enable { "enabled" } else { "disabled" }, interface);
		}
		Ok(())
	}

	pub async fn set_promiscuous_mode(&self, interface: &str, enable: bool) -> Result<(), NetworkError> {
		let index = self.get_interface_index(interface).await?;

		let mut link_message = LinkMessage::default();

		if enable {
			link_message.header.flags |= LinkFlags::Promisc
		} else {
			link_message.header.flags.remove(LinkFlags::Promisc)
		};
		link_message.header.change_mask |= LinkFlags::Promisc;
		link_message.header.index = index as u32;

		self.handle.link().set(link_message).execute().await?;
		info!("{} promiscuous mode on {}", if enable { "Enabled" } else { "Disabled" }, interface);
		Ok(())
	}

	pub async fn get_interface_status(&self, interface: &str) -> Result<InterfaceStatus, NetworkError> {
		let index = self.get_interface_index(interface).await?;

		let ipv6_status = self.get_ipv6_status(interface)?;
		let promisc_status = self.get_promiscuous_status(interface).await?;
		let qdisc_status = self.get_qdisc_status(index).await?;

		Ok(InterfaceStatus {
			name: interface.to_string(),
			ipv6_enabled: ipv6_status,
			promiscuous_mode: promisc_status,
			qdisc: qdisc_status,
		})
	}

	pub fn get_ipv6_status(&self, interface: &str) -> Result<bool, NetworkError> {
		let ctl_name = format!("net.ipv6.conf.{}.disable_ipv6", interface);
		let ctl = sysctl::Ctl::new(&ctl_name)
			.map_err(|_| NetworkError::SysctlError(format!("Could not get sysctl '{}'", ctl_name)))?;

		let value = ctl
			.value_string()
			.map_err(|_| NetworkError::SysctlError(format!("Could not get value for sysctl '{}'", ctl_name)))?;

		Ok(value == "0")
	}

	pub async fn get_promiscuous_status(&self, interface: &str) -> Result<bool, NetworkError> {
		let index = self.get_interface_index(interface).await?;
		let mut link = self.handle.link().get().execute();

		while let Ok(Some(link)) = link.try_next().await {
			if link.header.index == index as u32 {
				return Ok(link.header.flags.contains(LinkFlags::Promisc));
			}
		}

		Err(NetworkError::NotFound(format!("Could not find interface: {}", interface)))
	}
	pub async fn get_qdisc_status(&self, index: i32) -> Result<Vec<QdiscInfo>, NetworkError> {
		let mut qdiscs = self.handle.qdisc().get().execute();
		let mut status = Vec::new();

		while let Ok(Some(qdisc)) = qdiscs.try_next().await {
			if qdisc.header.index == index {
				status.push(self.parse_qdisc_message(qdisc));
			}
		}

		Ok(status)
	}

	pub fn parse_qdisc_message(&self, qdisc: TcMessage) -> QdiscInfo {
		let mut info = QdiscInfo {
			kind: String::new(),
			handle: format!("{:x}:", qdisc.header.handle.major),
			parent: format!("{:x}:", qdisc.header.parent.major),
			options: Vec::new(),
			stats: QdiscStats::default(),
		};

		for attr in qdisc.attributes {
			match attr {
				TcAttribute::Kind(kind) => info.kind = kind,
				TcAttribute::Options(opts) => {
					for opt in opts {
						info.options.push(format!("{:?}", opt));
					}
				}
				TcAttribute::Stats(stats) => {
					info.stats.bytes = stats.bytes;
					info.stats.packets = stats.packets as u64;
					info.stats.drops = stats.drops as u64;
					info.stats.overlimits = stats.overlimits as u64;
					info.stats.qlen = stats.qlen;
					info.stats.backlog = stats.backlog;
				}
				_ => {}
			}
		}

		info
	}
}
