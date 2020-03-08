```
# tar xf tor-0.4.blah blah

patch -d tor-tor-* -p0 < patches/tor-*

# remove those, otherwise cargo skips those folders while packing the crate
rm tor-tor-*/src/rust/Cargo.toml
rm tor-tor-*/src/rust/tor_rust/Cargo.toml

cd tor-tor-*
sh autogen.sh
cd ../
```

```
patch -d libevent-* -p0 < patches/libevent-*

cd libevent-*
sh autogen.sh
cd ../
```
