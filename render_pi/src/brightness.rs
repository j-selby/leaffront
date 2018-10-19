/// Manages the brightness of a Pi Touchscreen.
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use std::io;

pub fn set_brightness(brightness: u8) -> Result<(), io::Error> {
    let path = Path::new("/sys/class/backlight/rpi_backlight/brightness");

    let mut file = File::create(path)?;

    file.write(format!("{}", brightness).as_bytes())?;

    Ok(())
}
