//! Emit user messages and errors to R with [`Option`] and [`Result`] extensions.
use std::{
    ffi::{CStr, CString},
    fmt::Debug,
};

use libR_sys::{R_ShowMessage, Rf_error};

use crate::{
    prelude::*,
    sexp::{
        env::{Env, Symbol},
        function::Builtin,
    },
    ProtectedSEXP,
};

/// Extension trait for `Option` and `Result` to provide `unwrap_r` and `expect_r` methods.
pub trait UnwrapR {
    /// The output type of the `unwrap_r` and `expect_r` methods.
    type Output;
    /// Unwrap the value or ask R to throw an error.
    fn unwrap_r(self) -> Self::Output;
    /// Unwrap the value or ask R to throw an error with a custom message.
    fn expect_r(self, msg: &str) -> Self::Output;
}

impl<T> UnwrapR for Option<T> {
    type Output = T;

    fn unwrap_r(self) -> T {
        self.unwrap_or_else(|| r_error("unwrap_r called on None"))
    }

    fn expect_r(self, msg: &str) -> T {
        self.unwrap_or_else(|| r_error(msg))
    }
}

impl<T, E: Debug> UnwrapR for Result<T, E> {
    type Output = T;

    fn unwrap_r(self) -> T {
        self.unwrap_or_else(|e| r_error(&format!("unwrap_r called on Err: {:?}", e)))
    }

    fn expect_r(self, msg: &str) -> T {
        self.unwrap_or_else(|e| r_error(&format!("{}: {:?}", msg, e)))
    }
}

/// Emit an error message and stop execution.
pub fn r_error_c(msg: &CStr) -> ! {
    unsafe {
        Rf_error("%s\0".as_ptr() as *const i8, msg.as_ptr(), 0);
    };
}

/// Emit an error message and stop execution.
pub fn r_error(msg: &str) -> ! {
    let cstr = CString::new(msg).expect("Failed to convert message to CString");
    r_error_c(cstr.as_c_str());
}

/// Emit a message to the user.
pub fn r_message_c(msg: &CStr) {
    unsafe {
        R_ShowMessage(msg.as_ptr());
    }
}

/// Emit a message to the user.
pub fn r_message(msg: &str) {
    let cstr = CString::new(msg).expect("Failed to convert message to CString");
    r_message_c(cstr.as_c_str());
}

/// Flush the R console.
pub fn r_flush_console() {
    unsafe {
        libR_sys::R_FlushConsole();
    }
}

/// Get the last error message.
pub fn geterrmessage() -> Option<String> {
    Some(
        Env::base()
            .peek(Symbol::new_cstr(unsafe {
                CStr::from_bytes_with_nul_unchecked(b"geterrmessage\0")
            }))?
            .downcast_to::<Builtin<_>>()?
            .build_pairlist()
            .build_lang()
            .eval(Env::base())?
            .downcast_to::<CharacterVectorSEXP<_>>()?
            .get_elt(0)
            .as_str()?
            .to_string(),
    )
}
