//! # Build script

// Coding conventions
/*#![deny(non_upper_case_globals)]
#![deny(non_camel_case_types)]
#![deny(non_snake_case)]
#![deny(unused_mut)]
#![warn(missing_docs)]*/

extern crate autotools;
extern crate cc;

extern crate buildutils;

use std::env;
use std::fs;
use std::path::PathBuf;

use buildutils::*;

fn main() {
    //println!("cargo:version_number={:x}", openssl_version);

    /* for (key, value) in std::env::vars() {
        println!("{}: {}", key, value);
    } */

    // TODO https://github.com/arlolra/tor/blob/master/INSTALL#L32
    let event_dir = PathBuf::from(env::var("DEP_EVENT_ROOT").unwrap());
    let openssl_dir = PathBuf::from(env::var("DEP_OPENSSL_ROOT").unwrap());
    let mut zlib_dir = PathBuf::from(env::var("DEP_Z_ROOT").unwrap());
    zlib_dir.push("build");

    let full_version = env!("CARGO_PKG_VERSION");
    let path = source_dir(
        env!("CARGO_MANIFEST_DIR"),
        "tor-tor",
        &get_version(full_version),
    );
    let tor = autotools::Config::new(path.clone())
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
    //println!("{:?}", tor);

    println!(
        "cargo:rustc-link-search=native={}",
        openssl_dir.join("lib/").display()
    );
    println!("cargo:rustc-link-search=native={}", zlib_dir.display());

    println!(
        "cargo:rustc-link-search=native={}",
        tor.join("build/src/core").display()
    );
    println!(
        "cargo:rustc-link-search=native={}",
        tor.join("build/src/lib").display()
    );
    println!(
        "cargo:rustc-link-search=native={}",
        tor.join("build/src/trunnel").display()
    );
    println!(
        "cargo:rustc-link-search=native={}",
        tor.join("build/src/ext/ed25519/ref10").display()
    );
    println!(
        "cargo:rustc-link-search=native={}",
        tor.join("build/src/ext/ed25519/donna").display()
    );
    println!(
        "cargo:rustc-link-search=native={}",
        tor.join("build/src/ext/keccak-tiny").display()
    );

    println!("cargo:rustc-link-lib=static={}", "event");
    println!("cargo:rustc-link-lib=static={}", "event_pthreads");

    println!("cargo:rustc-link-lib=static={}", "crypto");
    println!("cargo:rustc-link-lib=static={}", "ssl");

    println!("cargo:rustc-link-lib=static={}", "z");

    println!("cargo:rustc-link-lib=static={}", "curve25519_donna");
    println!("cargo:rustc-link-lib=static={}", "ed25519_donna");
    println!("cargo:rustc-link-lib=static={}", "ed25519_ref10");
    println!("cargo:rustc-link-lib=static={}", "tor-confmgt");
    println!("cargo:rustc-link-lib=static={}", "tor-app");
    println!("cargo:rustc-link-lib=static={}", "keccak-tiny");
    println!("cargo:rustc-link-lib=static={}", "or-trunnel");
    println!("cargo:rustc-link-lib=static={}", "tor-intmath");
    println!("cargo:rustc-link-lib=static={}", "tor-lock");
    println!("cargo:rustc-link-lib=static={}", "tor-malloc");
    println!("cargo:rustc-link-lib=static={}", "tor-math");
    println!("cargo:rustc-link-lib=static={}", "tor-memarea");
    println!("cargo:rustc-link-lib=static={}", "tor-meminfo");
    println!("cargo:rustc-link-lib=static={}", "tor-osinfo");
    println!("cargo:rustc-link-lib=static={}", "tor-process");
    println!("cargo:rustc-link-lib=static={}", "tor-sandbox");
    println!("cargo:rustc-link-lib=static={}", "tor-smartlist-core");
    println!("cargo:rustc-link-lib=static={}", "tor-string");
    println!("cargo:rustc-link-lib=static={}", "tor-term");
    println!("cargo:rustc-link-lib=static={}", "tor-time");
    println!("cargo:rustc-link-lib=static={}", "tor-thread");
    println!("cargo:rustc-link-lib=static={}", "tor-wallclock");
    println!("cargo:rustc-link-lib=static={}", "tor-log");
    println!("cargo:rustc-link-lib=static={}", "tor-tls");
    println!("cargo:rustc-link-lib=static={}", "tor-compress");
    println!("cargo:rustc-link-lib=static={}", "tor-container");
    println!("cargo:rustc-link-lib=static={}", "tor-crypt-ops");
    println!("cargo:rustc-link-lib=static={}", "tor-ctime");
    println!("cargo:rustc-link-lib=static={}", "tor-encoding");
    println!("cargo:rustc-link-lib=static={}", "tor-net");
    println!("cargo:rustc-link-lib=static={}", "tor-err");
    println!("cargo:rustc-link-lib=static={}", "tor-evloop");
    println!("cargo:rustc-link-lib=static={}", "tor-fdio");
    println!("cargo:rustc-link-lib=static={}", "tor-fs");
    println!("cargo:rustc-link-lib=static={}", "tor-geoip");
    println!("cargo:rustc-link-lib=static={}", "tor-version");
    println!("cargo:rustc-link-lib=static={}", "tor-buf");
    println!("cargo:rustc-link-lib=static={}", "tor-pubsub");
    println!("cargo:rustc-link-lib=static={}", "tor-dispatch");
    println!("cargo:rustc-link-lib=static={}", "tor-trace");

    fs::create_dir_all(tor.join("include")).unwrap();
    fs::copy(
        path.join("src/feature/api/tor_api.h"),
        tor.join("include/tor_api.h"),
    )
    .unwrap();
    println!("cargo:include={}/include", tor.to_str().unwrap());

    // TODO: remove
    println!("cargo:rerun-if-changed=build.rs");
}
