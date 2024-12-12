use std::error::Error;

use net::{IfName, Nic};

fn main() -> Result<(), Box<dyn Error>> {
    match std::env::args().nth(1) {
        Some(ifname) => {
            let nic = Nic::default();

            let ifname: IfName = ifname.as_str().try_into()?;
            let lladdr = nic.get_lladd(&ifname)?;

            eprintln!("Nic.get_lladd({ifname}) -> {lladdr}");

            Ok(())
        }
        None => Err("Missing param: ifname".into()),
    }
}
