#[derive(Debug)]
pub struct QdiscInfo {
	pub(crate) kind: String,
	pub(crate) handle: String,
	pub(crate) parent: String,
	pub(crate) options: Vec<String>,
	pub(crate) stats: QdiscStats,
}

#[derive(Debug, Default)]
pub struct QdiscStats {
	pub(crate) bytes: u64,
	pub(crate) packets: u64,
	pub(crate) drops: u64,
	pub(crate) overlimits: u64,
	pub(crate) qlen: u32,
	pub(crate) backlog: u32,
}

pub struct InterfaceStatus {
	pub(crate) name: String,
	pub(crate) ipv6_enabled: bool,
	pub(crate) promiscuous_mode: bool,
	pub(crate) qdisc: Vec<QdiscInfo>,
}

impl std::fmt::Display for InterfaceStatus {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		writeln!(f, "Status of interface: {}", self.name)?;
		writeln!(f, "  IPv6:              {}", if self.ipv6_enabled { "Enabled" } else { "Disabled" })?;
		writeln!(f, "  Promiscuous mode:  {}", if self.promiscuous_mode { "Enabled" } else { "Disabled" })?;
		writeln!(f, "  Qdisc:")?;
		if self.qdisc.is_empty() {
			writeln!(f, "    None")?;
		} else {
			for qdisc in &self.qdisc {
				write!(f, "{}", qdisc)?;
			}
		}
		Ok(())
	}
}

impl std::fmt::Display for QdiscInfo {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		writeln!(f, "Qdisc {}, Handle {}, Parent {}", self.kind, self.handle, self.parent)?;
		if !self.options.is_empty() {
			writeln!(f, "  Options:")?;
			for opt in &self.options {
				writeln!(f, "    {}", opt)?;
			}
		}
		writeln!(f, "  Stats:")?;
		writeln!(f, "    Bytes: {}, Packets: {}", self.stats.bytes, self.stats.packets)?;
		writeln!(f, "    Drops: {}, Overlimits: {}", self.stats.drops, self.stats.overlimits)?;
		writeln!(f, "    Queue length: {}, Backlog: {}", self.stats.qlen, self.stats.backlog)
	}
}
