//! # Build script

// Coding conventions
/*#![deny(non_upper_case_globals)]
#![deny(non_camel_case_types)]
#![deny(non_snake_case)]
#![deny(unused_mut)]
#![warn(missing_docs)]*/

extern crate autotools;
extern crate cc;

use std::fs;
use std::env;
use std::path::PathBuf;

fn main() {
    //println!("cargo:version_number={:x}", openssl_version);

    /*for (key, value) in std::env::vars() {
        println!("{}: {}", key, value);
    } */

    // TODO https://github.com/arlolra/tor/blob/master/INSTALL#L32
    let event_dir = PathBuf::from(env::var("DEP_EVENT_ROOT").unwrap());
    let openssl_dir = PathBuf::from(env::var("DEP_OPENSSL_ROOT").unwrap());
    let mut zlib_dir = PathBuf::from(env::var("DEP_Z_ROOT").unwrap());
    zlib_dir.push("build");

    // Apply fix from https://github.com/Blockstream/gdk/blob/master/tools/buildtor.sh#L65
    // autogen.sh
    let tor = autotools::Config::new("tor")
        .with("libevent-dir", event_dir.to_str())
        .with("openssl-dir", openssl_dir.to_str())
        .with("zlib-dir", zlib_dir.to_str())
        .enable("pic", None)
        .enable("static-tor", None)
        .enable("static-openssl", None)
        .enable("static-libevent", None)
        .disable("system-torrc", None)
        .disable("asciidoc", None)
        .disable("systemd", None)
        .disable("zstd", None)
        .disable("lzma", None)
        .disable("largefile", None)
        .disable("unittests", None)
        .disable("tool-name-check", None)
        .disable("module-relay", None)
        .disable("rust", None)
        .build();
    println!("{:?}", tor);

    // package everything into one archive
    cc::Build::new()
        .object(tor.join("build/src/tools/libtorrunner.a"))
        .object(tor.join("build/src/core/libtor-app.a"))
        .object(tor.join("build/src/lib/libtor-compress.a"))
        .object(tor.join("build/src/lib/libtor-evloop.a"))
        .object(tor.join("build/src/lib/libtor-tls.a"))
        .object(tor.join("build/src/lib/libtor-crypt-ops.a"))
        .object(tor.join("build/src/ext/keccak-tiny/libkeccak-tiny.a"))
        .object(tor.join("build/src/lib/libcurve25519_donna.a"))
        .object(tor.join("build/src/ext/ed25519/ref10/libed25519_ref10.a"))
        .object(tor.join("build/src/ext/ed25519/donna/libed25519_donna.a"))
        .object(tor.join("build/src/lib/libtor-geoip.a"))
        .object(tor.join("build/src/lib/libtor-buf.a"))
        .object(tor.join("build/src/lib/libtor-process.a"))
        .object(tor.join("build/src/lib/libtor-confmgt.a"))
        .object(tor.join("build/src/lib/libtor-pubsub.a"))
        .object(tor.join("build/src/lib/libtor-dispatch.a"))
        .object(tor.join("build/src/lib/libtor-time.a"))
        .object(tor.join("build/src/lib/libtor-fs.a"))
        .object(tor.join("build/src/lib/libtor-encoding.a"))
        .object(tor.join("build/src/lib/libtor-sandbox.a"))
        .object(tor.join("build/src/lib/libtor-container.a"))
        .object(tor.join("build/src/lib/libtor-net.a"))
        .object(tor.join("build/src/lib/libtor-thread.a"))
        .object(tor.join("build/src/lib/libtor-memarea.a"))
        .object(tor.join("build/src/lib/libtor-math.a"))
        .object(tor.join("build/src/lib/libtor-meminfo.a"))
        .object(tor.join("build/src/lib/libtor-osinfo.a"))
        .object(tor.join("build/src/lib/libtor-log.a"))
        .object(tor.join("build/src/lib/libtor-lock.a"))
        .object(tor.join("build/src/lib/libtor-fdio.a"))
        .object(tor.join("build/src/lib/libtor-term.a"))
        .object(tor.join("build/src/lib/libtor-string.a"))
        .object(tor.join("build/src/lib/libtor-smartlist-core.a"))
        .object(tor.join("build/src/lib/libtor-malloc.a"))
        .object(tor.join("build/src/lib/libtor-wallclock.a"))
        .object(tor.join("build/src/lib/libtor-err.a"))
        .object(tor.join("build/src/lib/libtor-intmath.a"))
        .object(tor.join("build/src/lib/libtor-version.a"))
        .object(tor.join("build/src/lib/libtor-ctime.a"))
        .object(tor.join("build/src/trunnel/libor-trunnel.a"))
        .object(tor.join("build/src/lib/libtor-trace.a"))
        .compile("tor");

    fs::create_dir_all(tor.join("include")).unwrap();
    fs::copy("tor/src/feature/api/tor_api.h", tor.join("include/tor_api.h")).unwrap();
    println!("cargo:include={}/include", tor.to_str().unwrap());
}
