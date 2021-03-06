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
sudo apt-get install gcc-4.7-arm-linux-gnueabihf
rustup target add armv7-unknown-linux-gnueabihf
```

Several dependencies require a generic path to `gcc`, so a few ugly symlink
 *may* be required:

```bash
ln -s /usr/bin/arm-linux-gnueabihf-gcc-4.7 /usr/bin/arm-linux-gnueabihf-gcc
cp -r /usr/include/GL /usr/arm-linux-gnueabihf/include/GL
```

You are going to want to configure Cargo to find this linker in `~/.cargo/config`:

```toml
[target.armv7-unknown-linux-gnueabihf]
linker = "arm-linux-gnueabihf-gcc-4.7"
```

Finally, you are going to need access to the Raspberry Pi's OpenGLES/DispmanX/etc
 stack. This can be found on a Raspberry Pi at `/opt/vc/lib`, which is 
 the location which will be assumed from here on. These artifacts can also
 be found here (as of writing): <https://github.com/raspberrypi/firmware/tree/master/opt/vc/lib>

The supplied `station/build.sh` script will automatically invoke Cargo, strip
 the build artifact, and package it into a .deb file targeting
 `armv7-unknown-linux-gnueabihf`.

As Leaffront directly uses the the hardware video scaler and OpenGLES,
 a X server cannot run at the same time.

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