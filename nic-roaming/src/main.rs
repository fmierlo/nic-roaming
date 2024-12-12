use std::error::Error;

use net::Nic;

fn main() -> Result<(), Box<dyn Error>> {
    match std::env::args().nth(1) {
        Some(name) => {
            let nic = Nic::default();

            let lladd = nic.get_lladd(&name)?;

            eprintln!("Nic.get_lladd({name}) -> {lladd}");

            Ok(())
        }
        None => Err("Missing param: name".into()),
    }
}
