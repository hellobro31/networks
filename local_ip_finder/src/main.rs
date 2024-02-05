extern crate get_if_addrs;

fn main() {
    match get_if_addrs::get_if_addrs() {
        Ok(if_addrs) => {
            for iface in if_addrs {
                println!("{}: {}", iface.name, iface.addr.ip());
            }
        }
        Err(e) => println!("Failed to get local IP address: {}", e),
    }
}
