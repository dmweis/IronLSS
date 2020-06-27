use iron_lss;
use async_std::task::sleep;
use std::time::Duration;
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
    let mut driver = iron_lss::LSSDriver::new(&args.port).unwrap();
    driver.set_color(5, iron_lss::LedColor::Green).await?;
    driver.set_motion_profile(5, false).await?;
    driver.set_angular_holding(5, -4).await?;
    driver.set_angular_stiffness(5, -4).await?;
    driver.set_filter_position_count(5, 4).await?;
    for i in 0..10 {
        println!("Setting field to {}", i);
        // driver.set_filter_position_count(5, i).await?;
        for pos in 0_i32..90 {
            driver.move_to_position(5, pos as f32).await?;
            sleep(Duration::from_millis(20)).await;
        }
        for pos in (0_i32..90).rev() {
            driver.move_to_position(5, pos as f32).await?;
            sleep(Duration::from_millis(20)).await;
        }
    }
    Ok(())
}