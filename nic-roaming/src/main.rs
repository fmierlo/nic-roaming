use platform::nic;

fn main() {
    let name = "en0";

    let nic = nic::new();

    let mac_address = nic.get_mac_address(name);

    println!("nic={:?} mac_address={}", name, mac_address);
}
