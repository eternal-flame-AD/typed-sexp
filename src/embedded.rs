//! Embed R in Rust
use std::process::Command;

use libR_sys::*;

/// An embedded R instance.
pub struct EmbeddedR;

impl EmbeddedR {
    /// Initialize embedded R.
    pub unsafe fn init() -> Self {
        if std::env::var("R_HOME").is_err() {
            let out = Command::new("R")
                .arg("-s")
                .arg("-e")
                .arg("cat(normalizePath(R.home()))")
                .output()
                .expect("Failed to run R");
            let home = String::from_utf8(out.stdout).unwrap();
            std::env::set_var("R_HOME", home.trim());
        }
        if Rf_initialize_R(
            3,
            ["R\0".as_ptr(), "--slave\0".as_ptr(), "--silent\0".as_ptr()].as_ptr() as *mut *mut i8,
        ) != 0
        {
            panic!("Failed to initialize R");
        }
        R_CStackLimit = usize::MAX;
        setup_Rmainloop();

        EmbeddedR
    }
}

impl Drop for EmbeddedR {
    fn drop(&mut self) {
        unsafe {
            libR_sys::Rf_endEmbeddedR(0);
        }
    }
}
