use ipnetwork::IpNetwork;
use std::net::{IpAddr, Ipv4Addr};

fn subnet_calculator(target_ip: Ipv4Addr, cidr_notation: u8) -> Result<(), String> {
    let network = IpNetwork::new(IpAddr::V4(target_ip), cidr_notation)
        .map_err(|e| e.to_string())?;

    if let IpNetwork::V4(net) = network {
        let network_address = net.network();
        let broadcast_address = net.broadcast();
        let first_host_ip = net.nth(1).ok_or("No first host IP available")?;
        let size = net.size(); // This is u32 for IPv4Network
        let last_host_ip = net.nth(size - 2).ok_or("No last host IP available")?;
        let subnet_mask = net.mask();
        let wildcard_mask = !u32::from(subnet_mask) & 0xFFFFFFFF;

        println!("Network Address: {}", network_address);
        println!("Broadcast Address: {}", broadcast_address);
        println!("First Host IP: {}", first_host_ip);
        println!("Last Host IP: {}", last_host_ip);
        println!("Subnet Mask: {}", subnet_mask);
        println!("Wildcard Mask: {}", Ipv4Addr::from(wildcard_mask));
        println!("Number of Hosts: {}", size - 2);
    } else {
        return Err("IPv6 is not supported in this example".to_string());
    }

    Ok(())
}

fn main() {
    let target_ip = "192.168.1.0".parse().unwrap();
    let cidr_notation = 24;

    if let Err(e) = subnet_calculator(target_ip, cidr_notation) {
        eprintln!("Error calculating subnet: {}", e);
    }
}
