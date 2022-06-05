extern crate leaffront_core;
extern crate leaffront_weather;

#[cfg(feature = "raspberry_pi")]
extern crate leaffront_input_pi;
#[cfg(feature = "raspberry_pi")]
extern crate leaffront_render_pi;

#[cfg(feature = "glutin")]
extern crate leaffront_input_glutin;
#[cfg(feature = "glutin")]
extern crate leaffront_render_glutin;

#[cfg(feature = "null_backend")]
extern crate leaffront_backend_null;
#[cfg(feature = "redis_backend")]
extern crate leaffront_backend_redis;

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate toml;

extern crate clap;

extern crate image;

extern crate chrono;
extern crate rand;

extern crate ctrlc;

#[macro_use]
extern crate log;

mod config;
mod state;

mod background;
mod clock;
mod main_loop;

mod platform;

use clap::{Arg, Command};

use env_logger::Env;
use leaffront_core::version::VersionInfo;

use platform::*;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    let env = Env::default().default_filter_or("info");
    env_logger::init_from_env(env);

    let matches = Command::new("Leaffront")
        .version(VERSION)
        .author("Selby (https://github.com/j-selby)")
        .about("A simple photoframe for the Raspberry Pi")
        .long_about(
            "Leaffront uses DispmanX + OpenGL to provide a simple slideshow, \
             along with basic clock, date and weather information. \
             Most values can be configured, and is lightweight enough that other \
             applications can be run alongside to enhance the experience.",
        )
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .help("Provide a custom configuration file")
                .default_value("config.toml")
                .value_name("FILE")
                .required(false)
                .takes_value(true),
        )
        .arg(
            Arg::new("version")
                .short('v')
                .long("version")
                .help("Shows version information and exits.")
                .required(false),
        )
        .get_matches();

    if matches.is_present("version") {
        info!("Leaffront {}", VERSION);
        info!("Backend: {:?}", BackendImpl::version());
        info!("Input: {:?}", InputImpl::version());
        info!("Renderer: {:?}", DrawerImpl::version());
        return;
    }

    let config_file = matches.value_of("config").unwrap_or("config.toml");

    info!("Leaffront {}", VERSION);

    let config = config::load_config(config_file.into());

    main_loop::main_loop(config);
}
