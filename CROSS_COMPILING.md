# Android

Make sure you have the NDK toolchain in your `PATH`, i.e. add `<NDK>/toolchains/llvm/prebuilt/<host-tag>/bin/` to your `PATH` and then run the build with one of the `android` cargo targets like this:

```
cargo build -vv --features=vendored-openssl --target=aarch64-linux-android
```
# iOS

If you are on macOS and you have the `Xcode Command Line Tools` installed everything should work out of the box, both for `iphone` and `iphonesim`. You can start the build with:

```
cargo build -vv --features=vendored-openssl --target=aarch64-apple-ios
```

# Windows

To cross-compile for Windows from Linux install `mingw32` and/or `mingw64` and compile for the relevant cargo target, like:

```
cargo build -vv --features=vendored-openssl --target=x86_64-pc-windows-gnu
```
