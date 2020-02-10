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

use buildutils::*;

fn main() {
    // TODO: cmake on windows

    let target = env::var("TARGET").unwrap();
    let host = env::var("HOST").unwrap();

    let mut cc = cc::Build::new();
    cc.target(&target).host(&host);
    let compiler = cc.get_compiler();
    //println!("CC: {:?}", compiler.path());
    //return;

    let full_version = env!("CARGO_PKG_VERSION");
    let path = source_dir(
        env!("CARGO_MANIFEST_DIR"),
        "libevent",
        &get_version(full_version),
    );
    let mut config = autotools::Config::new(path);
    config
        .env("CC", compiler.path())
        .host(&target)
        .enable_static()
        .disable_shared()
        .with("pic", None)
        .disable("samples", None)
        .disable("openssl", None)
        .disable("libevent-regress", None)
        .disable("debug-mode", None)
        .disable("dependency-tracking", None);

    let libevent = config.build();
    let artifacts = Artifacts {
        lib_dir: libevent.join("lib"),
        include_dir: libevent.join("include"),
        libs: vec!["event".to_string(), "event_pthreads".to_string()], // TODO: on windows re-add the `lib` prefix
    };
    artifacts.print_cargo_metadata();
}
