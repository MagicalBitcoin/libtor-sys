```
# tar xf tor-0.4.blah blah

patch -d tor-src -p0 < patches/tor-*

# remove those, otherwise cargo skips those folders while packing the crate
rm tor-src/src/rust/Cargo.toml
rm tor-src/src/rust/tor_rust/Cargo.toml

cd tor-src
sh autogen.sh
cd ../
```

```
patch -d libevent-src -p0 < patches/libevent-*

cd libevent-src
sh autogen.sh
cd ../
```
