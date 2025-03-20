use anyhow::Result;
use log::info;

fn main() -> Result<()> {
    env_logger::init();

    info!("{}", i64::from_str_radix("555", 16)?);

    Ok(())
}
