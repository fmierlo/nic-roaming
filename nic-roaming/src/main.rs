use platform::nic::Nic;

fn main() {
    let name = std::env::args().nth(1).unwrap();

    let nic = Nic::default();

    let mac_address = nic.get_mac_address(&name);

    eprintln!("Nic.get_mac_address({name}) -> {mac_address}");
}
