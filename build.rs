extern crate autotools;
extern crate cc;
extern crate libtor_src;

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

macro_rules! std_env_expected {
    ($name:expr) => {{
        std::env::var($name).expect(&format!("{} expected", $name))
    }};
}

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

// pub fn autoreconf(path: &PathBuf) -> Result<(), Vec<u8>> {
//     match Command::new("autoreconf")
//         .current_dir(path)
//         .args(&["--force", "--install"])
//         .output()
//     {
//         Ok(output) => {
//             if !output.status.success() {
//                 Err(output.stderr)
//             } else {
//                 Ok(())
//             }
//         }
//         Err(e) => Err(format!("{:?}", e).as_bytes().to_vec()),
//     }
// }

fn build_libevent() -> Artifacts {
    // TODO: cmake on windows
    let target = std_env_expected!("TARGET");
    let host = std_env_expected!("HOST");

    let mut cc = cc::Build::new();
    cc.target(&target).host(&host);
    let compiler = cc.get_compiler();

    let root = PathBuf::from(env::var("OUT_DIR").expect("missing OUT_DIR")).join("libevent");
    fs::create_dir_all(&root).expect("Cannot write to `OUT_DIR`");

    let path = libtor_src::get_libevent_dir();

    // if let Err(e) = autoreconf(&path) {
    //     println!(
    //         "cargo:warning=Failed to run `autoreconf`: {:?}",
    //         String::from_utf8(e)
    //     );
    // }

    let mut config = autotools::Config::new(path.clone());
    config
        .out_dir(&root)
        .config_option("host", Some(&host))
        .env("CC", compiler.path())
        .env("CFLAGS", compiler.cflags_env())
        .enable_static()
        .disable_shared()
        .with("pic", None)
        .disable("samples", None)
        .disable("openssl", None)
        .disable("libevent-regress", None)
        .disable("debug-mode", None)
        .disable("dependency-tracking", None);

    let libevent = config.build();

    let mut libs = vec!["event".to_string()];
    if target.contains("windows") {
        // Statically link libssp-0.dll (part of mingw), otherwise it would have to be shipped
        // together with the executable
        println!("cargo:rustc-link-lib=static=ssp");
    } else {
        // Windows targets don't build event_pthreads
        libs.push("event_pthreads".to_string());
    }

    let artifacts = Artifacts {
        lib_dir: libevent.join("lib"),
        include_dir: root.join("include"),
        libs,
        root,
    };
    artifacts.print_cargo_metadata();

    artifacts
}

fn build_tor(libevent: Artifacts) {
    let target = std_env_expected!("TARGET");
    let host = std_env_expected!("HOST");

    let mut cc = cc::Build::new();
    cc.target(&target).host(&host);
    let compiler = cc.get_compiler();

    // for (key, value) in std::env::vars() {
    //     println!("{}: {}", key, value);
    // }
    // return;

    let openssl_dir = env::var("DEP_OPENSSL_ROOT").ok().map(PathBuf::from);
    let lzma_dir = env::var("DEP_LZMA_ROOT").ok().map(PathBuf::from);
    let zstd_dir = env::var("DEP_ZSTD_ROOT").ok().map(PathBuf::from);

    let path = libtor_src::get_tor_dir();

    // if let Err(e) = autoreconf(&path) {
    //     println!(
    //         "cargo:warning=Failed to run `autoreconf`: {:?}",
    //         String::from_utf8(e)
    //     );
    // }

    // lzma and zstd are enabled by default, but it doesn't fail if it can't find it
    let mut config = autotools::Config::new(path.clone());
    config
        .config_option("host", Some(&host))
        .env("CC", compiler.path())
        .with("libevent-dir", libevent.root.to_str())
        .enable("pic", None)
        //.enable("static-tor", None)
        .enable("static-libevent", None)
        .enable("static-zlib", None)
        .disable("system-torrc", None)
        .disable("asciidoc", None)
        .disable("systemd", None)
        .disable("largefile", None)
        .disable("unittests", None)
        .disable("tool-name-check", None)
        .disable("manpage", None)
        .disable("html-manual", None)
        .disable("module-dirauth", None)
        .disable("module-relay", None)
        .disable("module-dircache", None)
        .disable("seccomp", None)
        .disable("libscrypt", None)
        .disable("rust", None);
    let mut cflags = String::new();
    cflags += &format!(" {}", compiler.cflags_env().into_string().unwrap());

    if !cfg!(feature = "with-lzma") {
        config.disable("lzma", None);
    }
    if !cfg!(feature = "with-zstd") {
        config.disable("zstd", None);
    }

    if target.contains("windows") {
        // On Windows targets the configure script needs some extra libs so it properly detects OpenSSL
        config.env("LIBS", "-lcrypt32 -liphlpapi -lws2_32 -lgdi32");
    }

    if let Some(dir) = &openssl_dir {
        config
            .with("openssl-dir", dir.to_str())
            .enable("static-openssl", None);
    }
    if let Some(dir) = &lzma_dir {
        let lzma_include = std_env_expected!("DEP_LZMA_INCLUDE");

        config.env("LZMA_CFLAGS", format!("-I{}", lzma_include));
        config.env("LZMA_LIBS", dir.join("liblzma.a").to_str().unwrap());

        println!("cargo:rustc-link-lib=static={}", "lzma");
    }
    if let Some(dir) = &zstd_dir {
        let lzma_include = std_env_expected!("DEP_ZSTD_INCLUDE");

        config.env("ZSTD_CFLAGS", format!("-I{}", lzma_include));
        config.env("ZSTD_LIBS", dir.join("libzstd.a").to_str().unwrap());

        println!("cargo:rustc-link-lib=static={}", "zstd");
    }

    if target.contains("android") {
        // zlib is part of the `sysroot` on android. Use `clang` to get the full path so that we
        // can link with it.
        let output = compiler
            .to_command()
            .args(&["--print-file-name", "libz.a"])
            .output()
            .expect("Failed to run `clang`");
        if !output.status.success() {
            panic!("`clang` did not complete successfully");
        }
        let libz_path =
            std::str::from_utf8(&output.stdout).expect("Invalid path for `libz.a` returned");
        let libz_path = PathBuf::from(libz_path);
        let sysroot_lib = libz_path
            .parent()
            .expect("Invalid path for `libz.a` returned")
            .to_str()
            .unwrap();

        config
            .enable("android", None)
            .with("zlib-dir", Some(&sysroot_lib));

        println!("cargo:rustc-link-search=native={}", sysroot_lib);
    } else {
        let mut zlib_dir = PathBuf::from(std_env_expected!("DEP_Z_ROOT"));

        let zlib_include_dir = zlib_dir.join("include");
        cflags += &format!(" -I{}", zlib_include_dir.display());

        zlib_dir.push("lib");

        config.with("zlib-dir", zlib_dir.to_str());
        // .env("CFLAGS", format!("-I{}", zlib_include_dir.display()));

        println!("cargo:rustc-link-search=native={}", zlib_dir.display());
    }

    let tor = config.env("CFLAGS", cflags).build();

    if let Some(dir) = &openssl_dir {
        println!(
            "cargo:rustc-link-search=native={}",
            dir.join("lib/").display()
        );
    }
    println!(
        "cargo:rustc-link-search=native={}",
        tor.join("build/").display()
    );

    println!("cargo:rustc-link-lib=static={}", "event");
    if !target.contains("windows") {
        // Windows targets don't build event_pthreads
        println!("cargo:rustc-link-lib=static={}", "event_pthreads");
    }

    println!("cargo:rustc-link-lib=static={}", "z");

    println!("cargo:rustc-link-lib=static={}", "tor");

    if openssl_dir.is_some() {
        println!("cargo:rustc-link-lib=static={}", "crypto");
        println!("cargo:rustc-link-lib=static={}", "ssl");
    } else {
        println!("cargo:rustc-link-lib={}", "crypto");
        println!("cargo:rustc-link-lib={}", "ssl");
    }

    if target.contains("windows") {
        // println!("cargo:rustc-link-search=native=/usr/i686-w64-mingw32/lib");

        // Add the CC's library paths
        let output = Command::new(format!("{}", compiler.path().display()))
            .arg("-print-search-dirs")
            .output()
            .expect("CC doesn't accept -print-search-dirs");

        let output = std::str::from_utf8(&output.stdout).expect("Invalid output");
        let lines = output.lines().filter_map(|line| {
            if line.starts_with("libraries: =") {
                Some(line.replacen("libraries: =", "", 1))
            } else {
                None
            }
        });
        for line in lines {
            for path in line.split(':') {
                println!("cargo:rustc-link-search=native={}", path);
            }
        }

        println!("cargo:rustc-link-lib={}", "crypt32");
        println!("cargo:rustc-link-lib={}", "iphlpapi");
        println!("cargo:rustc-link-lib={}", "ws2_32");
        println!("cargo:rustc-link-lib={}", "gdi32");

        println!("cargo:rustc-link-lib={}", "shell32");
        println!("cargo:rustc-link-lib={}", "ssp");

        println!("cargo:rustc-link-lib={}", "shlwapi");
    }

    fs::create_dir_all(tor.join("include")).unwrap();
    fs::copy(
        path.join("src/feature/api/tor_api.h"),
        tor.join("include/tor_api.h"),
    )
    .unwrap();
    println!("cargo:include={}/include", tor.to_str().unwrap());

    println!("cargo:rerun-if-changed=build.rs");
}

fn main() {
    let libevent = build_libevent();
    build_tor(libevent);
}
