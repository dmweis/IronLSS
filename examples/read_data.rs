use iron_lss;
use clap::Clap;

#[derive(Clap)]
#[clap()]
struct Args {
    #[clap(
        about = "Serial port to use"
    )]
    port: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Args = Args::parse();
    let mut driver = iron_lss::LSSDriver::new(&args.port)?;
    println!("Voltage is {} V", driver.read_voltage(5).await?);
    println!("Temperature is {} °C", driver.read_temperature(5).await?);
    println!("Current is {} A", driver.read_current(5).await?);
    println!("Position is {} degrees", driver.read_position(5).await?);
    println!("Filter position count is {}", driver.read_filter_position_count(5).await?);
    Ok(())
}
