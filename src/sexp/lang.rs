use std::fmt::Debug;

use crate::{message::geterrmessage, prelude::*, DowncastSEXP, ProtectedSEXP};
use derive_more::Deref;
use libR_sys::*;

use super::env::Env;

#[derive(Deref)]
#[repr(transparent)]
/// A wrapper around a language object.
pub struct Lang<T: JustSEXP> {
    head: T,
}

impl<T: JustSEXP + Debug> Debug for Lang<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Lang").field(&self.head).finish()
    }
}

impl<T: JustSEXP> HasSEXP for Lang<T> {
    fn get_sexp(&self) -> SEXP {
        self.head.get_sexp()
    }
}

impl<T: JustSEXP> Lang<T> {
    /// Evaluate the language object in the given environment.
    pub fn eval<E: JustSEXP>(self, env: Env<E>) -> Option<SEXP> {
        unsafe {
            let mut error = 0;
            let result = R_tryEval(self.get_sexp(), env.get_sexp(), &mut error);
            if error != 0 {
                None
            } else {
                Some(result)
            }
        }
    }

    /// Evaluate the language object in the given environment, returning an error message if one occurs.
    ///
    /// This is a convenience method that calls [`Self::eval`] and then calls [`geterrmessage`] if an error occurs.
    pub fn try_eval<E: JustSEXP>(self, env: Env<E>) -> Result<SEXP, Option<String>> {
        self.eval(env).ok_or_else(geterrmessage)
    }
}

impl<T: JustSEXP> DowncastSEXP<Lang<T>> for T {
    fn downcast(self) -> Option<Lang<T>> {
        Lang::wrap_sexp(self.get_sexp())
    }
}

unsafe impl<T: JustSEXP> JustSEXP for Lang<T> {
    type Inner = T;
    fn inner_ref(&self) -> &Self::Inner {
        &self.head
    }
    unsafe fn wrap_sexp_unchecked(sexp: SEXP) -> Self {
        Lang {
            head: T::wrap_sexp_unchecked(sexp),
        }
    }
    fn wrap_sexp(sexp: SEXP) -> Option<Self> {
        if unsafe { Rf_isLanguage(sexp) }.into() {
            Some(Lang {
                head: T::wrap_sexp(sexp)?,
            })
        } else {
            None
        }
    }
    fn upcast(self) -> Self::Inner {
        self.head
    }
}

impl<T: JustSEXP> TypedSEXP for Lang<T> {
    const SEXP_TYPE: SEXPTYPE = SEXPTYPE::LANGSXP;
}

/// A builder for creating a pairlist.
pub struct PairlistBuilder<T: ProtectedSEXP> {
    head: T,
    cdr: Vec<(SEXP, Option<SEXP>)>,
}

impl<T: ProtectedSEXP> PairlistBuilder<T>
where
    <T as ProtectedSEXP>::Inner: JustSEXP,
{
    /// Create a new pairlist builder with the given head.
    pub fn new(head: T) -> Self {
        PairlistBuilder {
            head,
            cdr: Vec::new(),
        }
    }

    /// Push a pair onto the pairlist.
    pub fn push<U: HasSEXP + ProtectedSEXP>(mut self, sexp: U) -> Self {
        self.cdr.push((sexp.get_sexp(), None));
        self
    }

    /// Push a tagged pair onto the pairlist.
    pub fn push_tagged<U: HasSEXP + ProtectedSEXP, V: HasSEXP + ProtectedSEXP>(
        mut self,
        tag: U,
        sexp: V,
    ) -> Self {
        self.cdr.push((tag.get_sexp(), Some(sexp.get_sexp())));
        self
    }

    /// Build the pairlist into a [`Lang`].
    pub fn build_lang(self) -> Lang<<T as ProtectedSEXP>::Inner> {
        unsafe {
            let lang = Rf_allocLang((self.cdr.len() + 1) as _);
            Rf_protect(lang);
            let mut ptr = lang;
            SETCAR(ptr, self.head.get_sexp());
            ptr = CDR(ptr);
            for arg in self.cdr {
                SETCAR(ptr, arg.0);
                if let Some(tag) = arg.1 {
                    SET_TAG(ptr, tag);
                }
                ptr = CDR(ptr);
            }
            Rf_unprotect(1);
            Lang::wrap_sexp_unchecked(lang)
        }
    }
}
