```
# tar xf tor-0.4.blah blah

# remove those, otherwise cargo skips those folders while packing the crate
find tor-src -type f -name Cargo.toml -exec rm -vf '{}' \;
```
