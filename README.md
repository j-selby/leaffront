Leaffront
=========

[![Build Status](https://ci.jselby.net/job/leaffront/job/master/badge/icon)](https://ci.jselby.net/job/leaffront/job/master/)

A weather station and notification synchronisation platform, designed to run
 pretty much anywhere. Targeted at the Raspberry Pi.
 
![Splash](example.jpg)

Building
--------

Building the application should be fairly simple as long as you are compiling
 on the target platform.

```bash
cd station
cargo build --features glutin
```

Replace `glutin` with your preferred frontend.

Running
-------

Two things (other then the application binary) are required to run Leaffront
 in the working directory:

- An `art` directory, containing `.jpg`s or `.png`s. This can be empty, but
   Leaffront will only display a blank screen.
- A `config.toml` file. An example can be found [here](example_config.toml).

If you want to use Redis for notifications, you also going to need this installed
 and running. This can be found in the Debian package `redis-server`.

Cross-compilation (for the Raspberry Pi)
----------------------------------------

Cross-compiling *is* possible with the frontends provided (and is pretty much
 essential when targeting a platform such as the Pi), but requires a bit of
 hacking around.

Firstly, ensure that you have Rustup installed on your building machine, then
 you are going to want to install the required cross-compiling utilities
 (For Debian):

```bash
apt-get install build-essential gcc-aarch64-linux-gnu
rustup target add aarch64-unknown-linux-gnu
```

Using systemd
-------------

There is a systemd .service file provided for the Raspberry Pi, which can
 be found [here](res/leaffront.service). This runs as a system service, not
 a user one, and is only really useful in automated systems where there
 is no requirement for a X server.

The best way to use Leaffront on a traditional desktop platform would by
 via launching the executable from your DE.

D-Bus Notification support
--------------------------

The D-Bus bridge allows for messages sent via Gnome's Notification system,
 which is used by a whole array of GUI applications.

See [here](dbus/README.md) for more information.

Networked notifications
-----------------------

(TODO)

License
-------

Leaffront is licensed under the MIT license, which can be found [here](LICENSE).