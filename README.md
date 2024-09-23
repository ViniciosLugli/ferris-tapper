Here’s a README draft for your project **ferris-tapper**:

---

# ferris-tapper

This is a Rust-based network tap tool designed to set up a bridge between two networks, allowing traffic monitoring and analysis. This tool enables users to capture and inspect data flowing through network interfaces, making it ideal for tasks like reverse engineering, debugging, and network analysis.

## Features

-   **Bridge Configuration**: Easily set up a bridge between two network interfaces for packet capture and traffic monitoring.
-   **Traffic Mirroring**: Mirror network traffic between interfaces for real-time analysis.
-   **Qdisc Management**: Configure queuing disciplines (qdisc) to manage network traffic and prioritize flows.
    -   **IPv6**: Disable IPv6 on interfaces to focus on IPv4 traffic analysis and prevent IPv6-related issues.
    -   **Promiscuous**: Enable promiscuous mode on network interfaces to capture all traffic, not just traffic directed to the host, so the hosts can communicate with each other.
-   **Status**: Query the current status of network interfaces, including IPv6 settings, promiscuous mode, and qdisc statistics.

## Use Cases

-   **Network Monitoring**: Capture and analyze network traffic passing through the configured bridge.
-   **Reverse Engineering**: Inspect network protocols and interactions for reverse engineering purposes on devices or applications.
-   **Penetration Testing**: Use the tool in penetration testing scenarios to monitor and manipulate traffic.
-   **Debugging**: Identify and troubleshoot network issues by capturing packet-level information.

## Installation

Ensure you system is a **Linux** based, this project is designed for Linux systems, leveraging the `rtnetlink` and `sysctl` utilities for network configuration.

```bash
cargo install ferris-tapper
```

## Usage

Once installed, you can use `ferris-tapper` to set up, manage, and monitor network configurations.

> **Note**: Is recommended to run the tool with `sudo` or as `root` to ensure the necessary permissions for network configuration.

### Starting TAP Network Configuration

```bash
sudo ferris-tapper start eth0 eth1
```

This command sets up a bridge between `eth0` and `eth1`, enabling packet capture and analysis.

### Stopping TAP Network Configuration

```bash
sudo ferris-tapper stop eth0 eth1
```

Now, this command stops the bridge configuration between `eth0` and `eth1`, reverting the changes made by the tool.

### Querying Network Status/Configuration

```bash
sudo ferris-tapper status eth0 eth1
```

This command returns the current network configuration status, including IPv6 settings, qdisc information, and promiscuous mode status for each interface.

## License

This project is licensed under the **GPL-3.0**. See the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Feel free to submit issues or pull requests at: [GitHub - ferris-tapper](https://github.com/ViniciosLugli/ferris-tapper)

## Authors

-   **[Vinicios Lugli]** - [vinicioslugli@gmail.com](mailto:vinicioslugli@gmail.com)
