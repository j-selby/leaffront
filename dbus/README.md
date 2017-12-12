D-Bus Bridge
============

A bridge which converts D-Bus notifications, as defined
 [here](https://developer.gnome.org/notification-spec/), into Redis events
 which are then distributed through the normal mechanisms.

Building
--------

This is really the easiest to build on your target platform, so cross-compilation
 instructions are not provided.

```bash
cargo build
```

Alternatively, if you want a Debian package with a user systemd service (for the
 Raspberry Pi):

```bash
cargo deb
```

Testing
-------

While the bridge, Leaffront, and redis is running:

```bash
notify-send "Title" "Body"
```

should display a message to the screen. If this fails, make sure that
 you have access to the correct D-Bus session.
