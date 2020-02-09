use std::path::{Path, PathBuf};

pub fn source_dir(var: &str, package: &str, version: &str) -> PathBuf {
    Path::new(var).join(format!("{}-{}", package, version))
}

pub fn get_version(full_version: &str) -> String {
    let parts: Vec<_> = full_version.split('+').collect();
    parts[1].into()
}

pub struct Artifacts {
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
