# libtor-sys [![Build Status](https://travis-ci.org/MagicalBitcoin/libtor-sys.svg?branch=master)](https://travis-ci.org/MagicalBitcoin/libtor-sys)

This library compiles Tor and a few dependencies (zlib, libevent and openssl) into a single Rust library that can be imported like any other Rust crate into existing projects.
This provides a way to use Tor without having to ship/download extra binaries - on platforms that allows running them - while for some other platforms like
iOS and newer Android versions this is the only way to run Tor since the OS straight up doesn't allow exec'ing binaries.

Keep in mind that the interface exposed here is very very "low-level" (literally just what's in `tor_api.h`). Another crate wrapping all of these parts with a nice Rust interface will
be released separately.

## Supported platforms

The currently supported platforms are:

* Linux (tested on Fedora 30 and Ubuntu Xenial)
* Android through the NDK

Coming Soon :tm::

* MacOS (it *might* even work now, but I don't have a Mac to test it)
* iOS
* Windows

## Build gotchas

### Linux

Building on a Linux pc shouldn't be too hard, a `cargo build` should normally work. Keep in mind that you will need all the "usual" build tools, like a compiler, automake, autoconf, make, in your PATH.

If you get an `aclocal-1.15 not found` or something similar, try to cd into `libevent-sys/libevent-<version>` and run `autoreconf --force --install`. Repeat in `tor-tor-<version>` if you get the
same issue there, and then re-`cargo build`.

### Android

Cross-compiling for Android is a bit more complicated, there are a few things to adjust:
1. Some libraries (zlib) are provided by the OS, but we need to link against them. So an extra environment variable `SYSROOT` that points to the sysroot shipped with NDK is required to let Rust
know where to look for those libraries.
2. Rust by default will look for a compiler named `<arch>-linux-android-clang`, but that's not how they are called in the NDK. So a `CC` env variable should be provided, pointing to the right compiler
for the specific architecture.
3. Rust will also look for other compiling tools, like `ar`. Usually the best thing is to have the NDK's bin folder in your PATH so that it can find everything it needs.

At the end, your command line will look something like this, assuming you are building for `aarch64` on a `linux-x86_64` pc, targeting `android21`:

```
PATH="$PATH:$NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin" SYSROOT=$NDK_HOME/platforms/android-21/arch-arm64 CC="aarch64-linux-android21-clang" cargo build --target=aarch64-linux-android
```
