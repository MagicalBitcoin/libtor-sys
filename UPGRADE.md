```
# tar xf tor-0.4.blah blah

patch -d tor-tor-* -p0 < patches/tor-*

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
