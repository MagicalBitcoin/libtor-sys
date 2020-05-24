extern crate autotools;
extern crate cc;
extern crate fs_extra;

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use fs_extra::dir::{copy, remove, CopyOptions};

pub struct Artifacts {
    pub root: PathBuf,
    pub include_dir: PathBuf,
    pub lib_dir: PathBuf,
    pub libs: Vec<String>,
}

impl Artifacts {
    pub fn print_cargo_metadata(&self) {
        println!("cargo:rustc-link-search=native={}", self.lib_dir.display());
        for lib in self.libs.iter() {
            println!("cargo:rustc-link-lib=static={}", lib);
        }
        println!("cargo:include={}", self.include_dir.display());
        println!("cargo:lib={}", self.lib_dir.display());
    }
}

pub fn autoreconf(path: &PathBuf) -> Result<(), Vec<u8>> {
    match Command::new("autoreconf")
        .current_dir(path)
        .args(&["--force", "--install"])
        .output()
    {
        Ok(output) => {
            if !output.status.success() {
                Err(output.stderr)
            } else {
                Ok(())
            }
        }
        Err(e) => Err(format!("{:?}", e).as_bytes().to_vec()),
    }
}

fn build_libevent() -> Artifacts {
    // TODO: cmake on windows
    let target = env::var("TARGET").unwrap();
    let host = env::var("HOST").unwrap();

    let mut cc = cc::Build::new();
    cc.target(&target).host(&host);
    let compiler = cc.get_compiler();
    let root = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR expected")).join("libevent");
    fs::create_dir_all(&root).expect("Cannot create `libevent` in `OUT_DIR`");

    let original_src = Path::new(env!("CARGO_MANIFEST_DIR")).join("libevent-src");
    let path = Path::new(root.to_str().unwrap()).join("libevent-src");
    if path.exists() {
        remove(&path).expect("Unable to remove libevent's src folder");
    }
    copy(original_src, &root, &CopyOptions::new()).expect("Unable to copy libevent's src folder");

    if let Err(e) = autoreconf(&path) {
        println!(
            "cargo:warning=Failed to run `autoreconf`: {:?}",
            String::from_utf8(e)
        );
    }

    let mut config = autotools::Config::new(path.clone());
    config
        .out_dir(&root)
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
        include_dir: root.join("include"),
        libs: vec!["event".to_string(), "event_pthreads".to_string()], // TODO: on windows re-add the `lib` prefix
        root,
    };
    artifacts.print_cargo_metadata();

    artifacts
}

fn build_tor(libevent: Artifacts) {
    let target = env::var("TARGET").expect("TARGET expected");
    let host = env::var("HOST").expect("HOST expected");

    let mut cc = cc::Build::new();
    cc.target(&target).host(&host);
    let compiler = cc.get_compiler();

    /* for (key, value) in std::env::vars() {
        println!("{}: {}", key, value);
    }
    return; */

    // TODO https://github.com/arlolra/tor/blob/master/INSTALL#L32
    let openssl_dir =
        PathBuf::from(env::var("DEP_OPENSSL_ROOT").expect("DEP_OPENSSL_ROOT expected"));

    let original_src = Path::new(env!("CARGO_MANIFEST_DIR")).join("tor-src");
    let root = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR expected"));
    let path = PathBuf::from(root.to_str().unwrap()).join("tor-src");
    if path.exists() {
        remove(&path).expect("Unable to remove Tor's src folder");
    }
    copy(original_src, &root, &CopyOptions::new()).expect("Unable to copy Tor's src folder");

    if let Err(e) = autoreconf(&path) {
        println!(
            "cargo:warning=Failed to run `autoreconf`: {:?}",
            String::from_utf8(e)
        );
    }

    let mut config = autotools::Config::new(path.clone());
    config
        .env("CC", compiler.path())
        .with("libevent-dir", libevent.root.to_str())
        .cflag(format!("-I{}", libevent.include_dir.display()))
        .with("openssl-dir", openssl_dir.to_str())
        .enable("pic", None)
        //.enable("static-tor", None)
        .enable("static-openssl", None)
        .enable("static-libevent", None)
        .enable("static-zlib", None)
        .disable("system-torrc", None)
        .disable("asciidoc", None)
        .disable("systemd", None)
        .disable("zstd", None)
        .disable("lzma", None)
        .disable("largefile", None)
        .disable("unittests", None)
        .disable("tool-name-check", None)
        .disable("module-dirauth", None)
        .disable("seccomp", None)
        .disable("rust", None);

    if target.contains("android") {
        // Apparently zlib is already there on Android https://github.com/rust-lang/libz-sys/blob/master/build.rs#L42

        let sysroot_lib = format!("{}/usr/lib", env::var("SYSROOT").expect("SYSROOT expected"));

        // provides stdin and stderr
        cc::Build::new()
            .file("fake-stdio/stdio.c")
            .compile("libfakestdio.a");

        config
            .enable("android", None)
            .env(
                "LDFLAGS",
                format!(
                    "-L{} -L{}",
                    sysroot_lib,
                    env::var("OUT_DIR").expect("OUT_DIR expected")
                ),
            )
            .with("zlib-dir", Some(&sysroot_lib));

        println!("cargo:rustc-link-search=native={}", sysroot_lib);
    } else {
        let mut zlib_dir = PathBuf::from(env::var("DEP_Z_ROOT").expect("DEP_Z_ROOT expected"));
        let zlib_include_dir = zlib_dir.join("include");
        zlib_dir.push("build");

        config
            .with("zlib-dir", zlib_dir.to_str())
            .cflag(format!("-I{}", zlib_include_dir.display()));

        println!("cargo:rustc-link-search=native={}", zlib_dir.display());
    }

    let tor = config.build();

    println!(
        "cargo:rustc-link-search=native={}",
        openssl_dir.join("lib/").display()
    );
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

fn main() {
    let libevent = build_libevent();
    build_tor(libevent);
}
