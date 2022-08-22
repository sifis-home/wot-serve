use wot_serve::advertise::Advertiser;

use std::{thread::sleep, time::Duration};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ad = Advertiser::new()?;
    ad.add_service("lamp").build()?;

    println!("Advertising for 1 second");
    sleep(Duration::from_millis(1000));

    Ok(())
}
