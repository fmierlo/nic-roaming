use std::error::Error;

use net_sys::ifname::IfName;
use net_sys::lladdr::LLAddr;
use net_sys::nic;

#[cfg(not(tarpaulin_include))]
fn main() -> Result<(), Box<dyn Error>> {
    let action = std::env::args().nth(1);
    let ifname = std::env::args().nth(2);
    let lladdr = std::env::args().nth(3);

    match action.ok_or("Missing action param: [get | set]")?.as_str() {
        "get" => {
            let ifname: IfName = ifname.ok_or("Missing ifname param")?.try_into()?;
            let lladdr = nic::get_lladdr(&ifname)?;
            eprintln!("nic::get_lladdr({ifname}) -> {lladdr}");
        }
        "set" => {
            let ifname: IfName = ifname.ok_or("Missing ifname param")?.try_into()?;
            let lladdr: LLAddr = lladdr.ok_or("Missing lladdr param")?.parse()?;
            nic::set_lladdr(&ifname, &lladdr)?;
            eprintln!("nic::set_lladdr({ifname}, {lladdr})");
        }
        invalid => {
            return Err(format!("Invalid action: {invalid}").into());
        }
    }

    Ok(())
}
