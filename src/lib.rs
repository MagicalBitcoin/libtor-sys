use std::os::raw::{c_char, c_int, c_void};

type tor_main_configuration_t = c_void;

extern "C" {
    pub fn tor_main_configuration_new() -> *mut tor_main_configuration_t;
    pub fn tor_main_configuration_set_command_line(
        config: *mut tor_main_configuration_t,
        argc: c_int,
        argv: *const c_char,
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
            let mut config = tor_main_configuration_new();
            tor_main_configuration_set_command_line(
                config,
                1,
                CString::new("tor").unwrap().as_ptr(),
            );

            tor_run_main(config);
        }
    }
}
