use wot_serve::advertise::Advertiser;

use std::{thread::sleep, time::Duration};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ad = Advertiser::new()?;
    ad.add_service("lamp").build()?;

    loop {
        sleep(Duration::from_millis(100));
    }
}
