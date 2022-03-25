# libtor-src

This crate contains the source code for Tor and Libevent, and a set of patches that are applied in the build script to prepare for compilation of `libtor-sys`.

The patches are applied by default using `git`, but enabling the `use-gnu-patch` feature will make the build script using `patch` instead.
