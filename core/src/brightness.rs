//! Generic brightness controls.

use std::{path::Path, fs::File, io::{Write, self}};

pub fn set_brightness(brightness: u8) -> Result<(), io::Error> {
    let path = Path::new("/sys/class/backlight/rpi_backlight/brightness");

    if path.exists() {
        println!("Setting brightness using RPI Backlight endpoint...");
        let mut file = File::create(path)?;

        file.write(format!("{}", brightness).as_bytes())?;

        return Ok(());
    }

    println!("No brightness control available for this platform.");

    Ok(())
}
