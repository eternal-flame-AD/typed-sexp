#![allow(dead_code)]
use libR_sys::SEXP;

pub(crate) static mut PROTECT_STACK_CHECK: CheckedProtectStack = CheckedProtectStack::new();

pub(crate) struct CheckedProtectStack {
    seen_sexps: Vec<SEXP>,
}

impl CheckedProtectStack {
    pub(crate) const fn new() -> Self {
        CheckedProtectStack {
            seen_sexps: Vec::new(),
        }
    }

    pub(crate) fn push(&mut self, sexp: SEXP) {
        self.seen_sexps.push(sexp);
    }

    pub(crate) fn checked_pop(&mut self, sexp: SEXP) {
        if let Some(last) = self.seen_sexps.pop() {
            assert_eq!(last, sexp, "Incorrect SEXP popped from protect stack");
        } else {
            panic!("Unbalanced protect/unprotect calls");
        }
    }
}
