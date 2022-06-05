//! Generic brightness controls.

use std::{
    fs::{self, File},
    io::{self, Write},
    path::Path,
};

pub fn set_brightness(brightness: u8) -> Result<(), io::Error> {
    // Attempt to search for a brightness controller
    let path = Path::new("/sys/class/backlight/");

    if path.exists() {
        let path_contents = fs::read_dir(path)?;
        for child in path_contents {
            let child = child?;
            let child_metadata = child.metadata()?;

            if child_metadata.is_dir() || child_metadata.is_symlink() {
                // This is a possibility!
                let mut child_path = child.path();
                println!("Found screen controller classed object: {:?}", child_path);
                child_path.push("brightness");

                if child_path.exists() {
                    println!("Found brightness endpoint.");

                    let mut file = File::create(path)?;

                    file.write(format!("{}", brightness).as_bytes())?;

                    return Ok(());
                }
            }
        }
    } else {
        println!("Platform doesn't support Linux-style /sys endpoints");
    }

    println!("No brightness control available for this platform.");

    Ok(())
}
