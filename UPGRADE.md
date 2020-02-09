```
# Apply fix from https://github.com/Blockstream/gdk/blob/master/tools/buildtor.sh#L65

patch -p0 < patches/0001-linux-fix-openssl.patch
cd tor-tor-* && ./autogen.sh && cd ../
```

then remove `/configure` from tor-tor-*/.gitignore
