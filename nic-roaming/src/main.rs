use std::error::Error;

use platform::nic::Nic;

fn main() -> Result<(), Box<dyn Error>> {
    match std::env::args().nth(1) {
        Some(name) => {
            let nic = Nic::default();

            let mac_address = nic.get_mac_address(&name)?;

            eprintln!("Nic.get_mac_address({name}) -> {mac_address}");
            Ok(())
        }
        None => Err("Missing param: name".into()),
    }
}
