# libtor-sys ![Continuous integration](https://github.com/MagicalBitcoin/libtor-sys/workflows/Continuous%20integration/badge.svg?branch=master)

This library compiles Tor and a few dependencies (zlib, libevent and openssl) into a single Rust library that can be imported like any other Rust crate into existing projects.
This provides a way to use Tor without having to ship/download extra binaries - on platforms that allows running them - while for some other platforms like
iOS and newer Android versions this is the only way to run Tor since the OS straight up doesn't allow exec'ing binaries.

Keep in mind that the interface exposed here is very very "low-level" (literally just what's in `tor_api.h`). Another crate wrapping all of these parts with a nice Rust interface will
be released separately.

## Supported platforms

The currently supported platforms are:

* Linux (tested on Fedora 30 and Ubuntu Xenial)
* Android through the NDK
* MacOS
* iOS
* Windows cross-compiled from Linux with `mingw`

Coming Soon :tm::

* Windows (natively built)

## Build gotchas

Command examples to cross-compile for multiple platforms are available in [`CROSS_COMPILING.md`](CROSS_COMPILING.md).

### Linux/macOS

Building on a UNIX-like os shouldn't be too hard, a `cargo build` should normally work. Keep in mind that you will need all the "usual" build tools, like a compiler, automake, autoconf, make, in your PATH. On macOS
you can install those tools using `brew`.

If you get an `aclocal-1.15 not found` or something similar, try to cd into `libevent-src` and run `autoreconf --force --install`. Repeat in `tor-src` if you get the
same issue there, and then re-`cargo build`.

### Android

Cross-compiling for Android is fairly straightforward, just make sure that you have the NDK toolchain in your `PATH`. If you do so, `cargo build` will use the right compiler targeting the minimum supported
sdk version of the NDK you are using (generally 16 for `armv7` and 21 for everything else).

### iOS

Cross-compiling for iOS on a Mac that has the `Xcode Command Line Tools` installed should work out of the box.

### Windows (MingW)

Cross-compiling for Windows using MingW should also work out of the box, as long as you have the right compiler and the required libraries installed. To link the library into binaries it's generally required to also
install the static version of `libwinpthreads`.
