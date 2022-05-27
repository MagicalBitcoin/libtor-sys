extern crate fs_extra;

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use fs_extra::dir::{copy, remove, CopyOptions};

macro_rules! wrap_command {
    ($cmd:expr, $args:expr, $cwd:expr) => {{
        match Command::new($cmd).current_dir($cwd).args($args).output() {
            Ok(output) => {
                if !output.status.success() {
                    Err(String::from_utf8_lossy(&output.stderr).into_owned())
                } else {
                    Ok(())
                }
            }
            Err(e) => Err(format!("{:?}", e)),
        }
    }};
}

fn autogen(path: &Path) -> Result<(), String> {
    wrap_command!("sh", &["autogen.sh"], path)
}

#[cfg(feature = "use-gnu-patch")]
fn apply_patch(patch: &Path, target: &Path) -> Result<(), String> {
    wrap_command!(
        "patch",
        &["-p0", "-i", &patch.display().to_string()],
        target
    )
}

#[cfg(feature = "use-git-apply")]
fn apply_patch(patch: &Path, target: &Path) -> Result<(), String> {
    wrap_command!(
        "git",
        &["apply", "-p1", &patch.display().to_string()],
        target
    )
}

fn copy_src(name: &str) -> PathBuf {
    let original_src = Path::new(env!("CARGO_MANIFEST_DIR")).join(name);
    let root = PathBuf::from(env::var("OUT_DIR").expect("missing OUT_DIR")).join(name);

    fs::create_dir_all(&root).expect("Cannot write to `OUT_DIR`");

    let path = Path::new(root.to_str().unwrap()).join(name);
    if path.exists() {
        remove(&path).expect("Unable to remove src folder");
    }
    copy(original_src, &root, &CopyOptions::new()).expect("Unable to copy src folder");

    path.into()
}

fn get_patches(prefix: &str) -> Vec<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("patches");

    fs::read_dir(&root)
        .expect("Unable to list patches")
        .collect::<Result<Vec<_>, _>>()
        .expect("Error listing patches")
        .into_iter()
        .filter(|entry| {
            entry
                .file_name()
                .into_string()
                .expect("Invalid patch name")
                .starts_with(prefix)
        })
        .map(|entry| entry.path().into())
        .collect()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let libevent = copy_src("libevent-src");
    let tor = copy_src("tor-src");

    for patch in get_patches("libevent") {
        apply_patch(&patch, &libevent)
            .map_err(|e| format!("Error applying patch '{}': {:?}", patch.display(), e))?;
    }
    for patch in get_patches("tor") {
        apply_patch(&patch, &tor)
            .map_err(|e| format!("Error applying patch '{}': {:?}", patch.display(), e))?;
    }

    autogen(&libevent)?;
    autogen(&tor)?;

    println!("cargo:rustc-env=LIBEVENT_SRC={}", libevent.display());
    println!("cargo:rustc-env=TOR_SRC={}", tor.display());

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=tor-src");
    println!("cargo:rerun-if-changed=libevent-src");

    Ok(())
}
