use net::{IfName, LLAddr, Nic};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let action = std::env::args().nth(1);
    let ifname = std::env::args().nth(2);
    let lladdr = std::env::args().nth(3);

    match action.ok_or("Missing action param: [get | set]")?.as_str() {
        "get" => {
            let ifname: IfName = ifname.ok_or("Missing ifname param")?.try_into()?;
            let lladdr = Nic::default().get_lladd(&ifname)?;
            eprintln!("Nic.get_lladd({ifname}) -> {lladdr}");
        }
        "set" => {
            let ifname: IfName = ifname.ok_or("Missing ifname param")?.try_into()?;
            let lladdr: LLAddr = lladdr.ok_or("Missing lladdr param")?.parse()?;
            Nic::default().set_lladd(&ifname, &lladdr)?;
            eprintln!("Nic.set_lladd({ifname}, {lladdr})");
        }
        invalid => {
            return Err(format!("Invalid action: {invalid}").into());
        }
    }

    Ok(())
}
