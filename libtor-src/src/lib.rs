use std::path::{Path, PathBuf};

pub fn get_libevent_dir() -> PathBuf {
    Path::new(env!("LIBEVENT_SRC")).into()
}

pub fn get_tor_dir() -> PathBuf {
    Path::new(env!("TOR_SRC")).into()
}
