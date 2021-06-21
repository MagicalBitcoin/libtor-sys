```
# tar xf tor-0.4.blah blah

patch -d tor-src -p0 < patches/tor-*

# remove those, otherwise cargo skips those folders while packing the crate
find tor-src -type f -name Cargo.toml -exec rm -vf '{}' \;

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
