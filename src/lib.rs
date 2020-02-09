#![allow(non_camel_case_types)]

use std::os::raw::{c_char, c_int, c_void};

type tor_main_configuration_t = c_void;

extern "C" {
    pub fn tor_main_configuration_new() -> *mut tor_main_configuration_t;
    pub fn tor_main_configuration_set_command_line(
        config: *mut tor_main_configuration_t,
        argc: c_int,
        argv: *const *const c_char,
    ) -> c_int;
    pub fn tor_run_main(configuration: *const tor_main_configuration_t) -> c_int;
}

#[cfg(test)]
mod tests {
    use crate::*;
    use std::ffi::CString;

    #[test]
    fn test_start() {
        unsafe {
            let config = tor_main_configuration_new();
            let argv = vec![
                CString::new("tor").unwrap(),
                CString::new("--version").unwrap(),
            ];
            let argv: Vec<_> = argv.iter().map(|s| s.as_ptr()).collect();
            tor_main_configuration_set_command_line(config, argv.len() as i32, argv.as_ptr());

            tor_run_main(config);
        }
    }
}
