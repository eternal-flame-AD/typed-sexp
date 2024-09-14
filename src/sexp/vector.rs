use std::{
    ffi::CStr,
    fmt::{Debug, Display},
    ops::{Index, IndexMut},
};

use libR_sys::{SEXPTYPE::*, *};

use crate::{DowncastSEXP, HasSEXP, IndexableSEXP, JustSEXP, TypedSEXP};
use derive_more::Deref;

macro_rules! impl_simple_vector {
    ($struct:ident, $sexptype:ident,
            $elem_ty:ty,
            $scalar_constructor:ident, $deref_fn:ident,
            $elt_fn:ident, $set_elt_fn:ident) => {
        #[repr(transparent)]
        #[derive(Deref)]
        /// A wrapper around a vector of a $sexptype.
        pub struct $struct<T: HasSEXP> {
            inner: T,
        }

        impl $struct<SEXP> {
            #[must_use]
            /// Create a new vector of length `len`.
            pub fn new(len: usize) -> Self {
                unsafe {
                    let sexp = Rf_allocVector($sexptype, len as R_xlen_t);
                    $struct { inner: sexp }
                }
            }

            #[must_use]
            /// Check if the given `SEXP` is a vector of this type.
            pub fn sexp_is_this_type(sexp: SEXP) -> bool {
                unsafe { TYPEOF(sexp) == $sexptype }
            }

            #[must_use]
            /// Check if the `SEXP` wrapped in this struct is a vector of the correct type.
            pub fn correct_type(&self) -> bool {
                Self::sexp_is_this_type(self.inner.get_sexp())
            }

            #[must_use]
            /// Create a scalar vector.
            pub fn scalar(value: $elem_ty) -> Self {
                unsafe {
                    let sexp = $scalar_constructor(value);
                    $struct { inner: sexp }
                }
            }
        }

        impl<T: HasSEXP> $struct<T> {
            #[must_use]
            /// Upcast this vector to the inner type.
            pub fn upcast(self) -> T {
                self.inner
            }

            #[must_use]
            /// Get a slice of the elements in this vector.
            pub fn as_slice(&self) -> &[$elem_ty] {
                unsafe {
                    let sexp = self.inner.get_sexp();
                    std::slice::from_raw_parts($deref_fn(sexp), Rf_xlength(sexp) as usize)
                }
            }
        }

        impl<T: HasSEXP> IndexableSEXP for $struct<T> {
            type Index = usize;
            type Output = $elem_ty;

            fn len(&self) -> usize {
                unsafe { Rf_xlength(self.inner.get_sexp()) as usize }
            }

            fn get_elt(&self, index: usize) -> Self::Output {
                self.check_inbound(index);
                unsafe {
                    let sexp = self.inner.get_sexp();
                    $deref_fn(sexp).add(index).read()
                }
            }

            fn set_elt(&mut self, index: usize, value: impl Into<Self::Output>) {
                self.check_inbound(index);
                unsafe {
                    let sexp = self.inner.get_sexp();
                    $set_elt_fn(sexp, index as R_xlen_t, value.into());
                }
            }
        }

        impl<T: HasSEXP> Index<usize> for $struct<T> {
            type Output = $elem_ty;

            fn index(&self, index: usize) -> &Self::Output {
                self.check_inbound(index);
                unsafe {
                    let sexp = self.inner.get_sexp();
                    &*$deref_fn(sexp).add(index)
                }
            }
        }

        impl<T: HasSEXP> IndexMut<usize> for $struct<T> {
            fn index_mut(&mut self, index: usize) -> &mut Self::Output {
                self.check_inbound(index);
                unsafe {
                    let sexp = self.inner.get_sexp();
                    &mut *$deref_fn(sexp).add(index)
                }
            }
        }

        impl<T: HasSEXP> DowncastSEXP<$struct<T>> for T {
            fn downcast(self) -> Option<$struct<T>> {
                if unsafe { TYPEOF(self.get_sexp()) } == $sexptype {
                    Some($struct { inner: self })
                } else {
                    None
                }
            }
        }

        impl<T: HasSEXP> HasSEXP for $struct<T> {
            fn get_sexp(&self) -> SEXP {
                self.inner.get_sexp()
            }
        }

        impl<T: HasSEXP> TypedSEXP for $struct<T> {
            const SEXP_TYPE: SEXPTYPE = $sexptype;
        }

        impl<T: HasSEXP> Debug for $struct<T> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(
                    f,
                    "A {} vector of length {}",
                    stringify!($sexptype),
                    self.len()
                )
            }
        }

        impl<T: HasSEXP> Display for $struct<T> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "[")?;
                for i in 0..self.len() {
                    write!(f, "{:?}", self[i])?;
                    if i != self.len() - 1 {
                        write!(f, ", ")?;
                    }
                }
                write!(f, "]")
            }
        }

        unsafe impl<T: JustSEXP> JustSEXP for $struct<T> {
            type Inner = T;

            fn upcast(self) -> Self::Inner {
                self.inner
            }

            fn wrap_sexp(sexp: SEXP) -> Option<Self> {
                if unsafe { TYPEOF(sexp) } == $sexptype {
                    Some($struct {
                        inner: T::wrap_sexp(sexp)?,
                    })
                } else {
                    None
                }
            }

            fn inner_ref(&self) -> &Self::Inner {
                &self.inner
            }

            unsafe fn wrap_sexp_unchecked(sexp: SEXP) -> Self {
                $struct {
                    inner: T::wrap_sexp_unchecked(sexp),
                }
            }
        }
    };
}

impl_simple_vector!(
    IntegerVector,
    INTSXP,
    i32,
    Rf_ScalarInteger,
    INTEGER,
    INTEGER_ELT,
    SET_INTEGER_ELT
);
impl_simple_vector!(
    RealVector,
    REALSXP,
    f64,
    Rf_ScalarReal,
    REAL,
    REAL_ELT,
    SET_REAL_ELT
);
impl_simple_vector!(
    LogicalVector,
    LGLSXP,
    i32,
    Rf_ScalarLogical,
    LOGICAL,
    LOGICAL_ELT,
    SET_LOGICAL_ELT
);

impl LogicalVector<SEXP> {
    #[must_use]
    /// Create a scalar logical vector.
    pub fn scalar_bool(value: bool) -> Self {
        Self::scalar(value as i32)
    }
}

/// A wrapper around a vector of strings.
#[derive(Deref)]
pub struct CharacterVector<T: HasSEXP> {
    inner: T,
}

impl CharacterVector<SEXP> {
    /// Create a new vector of strings of length `len`.
    #[must_use]
    pub fn new(len: usize) -> Self {
        unsafe {
            let sexp = Rf_allocVector(STRSXP, len as R_xlen_t);
            CharacterVector { inner: sexp }
        }
    }

    /// Create a scalar vector of strings.
    #[must_use]
    pub fn scalar(value: &str) -> Self {
        unsafe {
            let sexp = Rf_mkCharLenCE(
                value.as_ptr() as *const i8,
                value.len() as _,
                cetype_t::CE_UTF8,
            );
            let str_sexp = Rf_ScalarString(sexp);
            CharacterVector { inner: str_sexp }
        }
    }
}

impl<T: HasSEXP> CharacterVector<T> {
    #[must_use]
    /// Check if the given `SEXP` is a vector of strings.
    pub fn sexp_is_this_type(sexp: SEXP) -> bool {
        unsafe { TYPEOF(sexp) == STRSXP }
    }
    #[must_use]
    /// Check if the `SEXP` wrapped in this struct is a vector of strings.
    pub fn correct_type(&self) -> bool {
        Self::sexp_is_this_type(self.inner.get_sexp())
    }
    /// Set the string at the given index.
    pub fn set_str(&mut self, index: usize, value: impl Into<Character>) {
        unsafe {
            SET_STRING_ELT(self.get_sexp(), index as R_xlen_t, value.into().get_sexp());
        }
    }
}

impl<T: HasSEXP> IndexableSEXP for CharacterVector<T> {
    type Index = usize;
    type Output = Character;
    fn len(&self) -> usize {
        unsafe { Rf_xlength(self.inner.get_sexp()) as usize }
    }

    fn get_elt(&self, index: usize) -> Self::Output {
        unsafe {
            let sexp = self.inner.get_sexp();
            let ptr = STRING_ELT(sexp, index as R_xlen_t);
            Character::wrap_sexp_unchecked(ptr)
        }
    }

    fn set_elt(&mut self, index: usize, value: impl Into<Self::Output>) {
        unsafe {
            let sexp = self.inner.get_sexp();
            SET_STRING_ELT(sexp, index as R_xlen_t, value.into().get_sexp());
        }
    }
}

impl<T: HasSEXP> HasSEXP for CharacterVector<T> {
    fn get_sexp(&self) -> SEXP {
        self.inner.get_sexp()
    }
}

impl<T: HasSEXP> TypedSEXP for CharacterVector<T> {
    const SEXP_TYPE: SEXPTYPE = STRSXP;
}

impl<T: HasSEXP> DowncastSEXP<CharacterVector<T>> for T {
    fn downcast(self) -> Option<CharacterVector<T>> {
        if unsafe { TYPEOF(self.get_sexp()) } == STRSXP {
            Some(CharacterVector { inner: self })
        } else {
            None
        }
    }
}

impl<T: HasSEXP> Debug for CharacterVector<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "A character vector of length {}", self.len())
    }
}

impl<T: HasSEXP> Display for CharacterVector<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        for i in 0..self.len() {
            write!(f, "{:?}", self.get_elt(i))?;
            if i != self.len() - 1 {
                write!(f, ", ")?;
            }
        }
        write!(f, "]")
    }
}

unsafe impl<T: JustSEXP> JustSEXP for CharacterVector<T> {
    type Inner = T;

    fn upcast(self) -> Self::Inner {
        self.inner
    }

    fn inner_ref(&self) -> &Self::Inner {
        &self.inner
    }

    fn wrap_sexp(sexp: SEXP) -> Option<Self> {
        if unsafe { TYPEOF(sexp) } == STRSXP {
            Some(CharacterVector {
                inner: T::wrap_sexp(sexp)?,
            })
        } else {
            None
        }
    }

    unsafe fn wrap_sexp_unchecked(sexp: SEXP) -> Self {
        CharacterVector {
            inner: T::wrap_sexp_unchecked(sexp),
        }
    }
}

#[derive(Deref)]
#[repr(transparent)]
/// A wrapper around a CHARSXP.
pub struct Character {
    sexp: SEXP,
}

impl HasSEXP for Character {
    fn get_sexp(&self) -> SEXP {
        self.sexp
    }
}

impl TypedSEXP for Character {
    const SEXP_TYPE: SEXPTYPE = CHARSXP;
}

unsafe impl JustSEXP for Character {
    type Inner = SEXP;

    fn upcast(self) -> Self::Inner {
        self.sexp
    }

    fn inner_ref(&self) -> &Self::Inner {
        &self.sexp
    }

    fn wrap_sexp(sexp: SEXP) -> Option<Self> {
        if unsafe { TYPEOF(sexp) } == CHARSXP {
            Some(Character { sexp })
        } else {
            None
        }
    }

    unsafe fn wrap_sexp_unchecked(sexp: SEXP) -> Self {
        Character { sexp }
    }
}

impl Character {
    /// Create a new character vector. Note this is just a single string, not a string vector.
    pub fn new(value: &str) -> Self {
        unsafe {
            let sexp = Rf_mkCharLenCE(
                value.as_ptr() as *const i8,
                value.len() as _,
                cetype_t::CE_UTF8,
            );
            Character { sexp }
        }
    }
    /// Try to convert the character vector to a UTF-8 string.
    pub fn as_str(&self) -> Option<&str> {
        unsafe {
            let out = Rf_translateCharUTF8(self.sexp);
            CStr::from_ptr(out).to_str().ok()
        }
    }
}

impl Debug for Character {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "'{:?}'", self.as_str())
    }
}

impl Display for Character {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str().unwrap_or_default())
    }
}

impl TryInto<String> for Character {
    type Error = &'static str;

    fn try_into(self) -> Result<String, Self::Error> {
        self.as_str().map(ToOwned::to_owned).ok_or("Invalid UTF-8")
    }
}

impl<S: AsRef<str>> From<S> for Character {
    fn from(value: S) -> Self {
        Character::new(value.as_ref())
    }
}

#[derive(Deref)]
#[repr(transparent)]
/// A wrapper around a raw vector (VECSXP, called list in R).
pub struct List<T: HasSEXP> {
    sexp: T,
}

impl<T: HasSEXP + Debug> Debug for List<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "List of length {}", self.len())
    }
}

impl<T: HasSEXP + Display> Display for List<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        for i in 0..self.len() {
            write!(f, "{:?}", self.get_elt(i))?;
            if i != self.len() - 1 {
                write!(f, ", ")?;
            }
        }
        write!(f, "]")
    }
}

impl List<SEXP> {
    /// Create a new list of length `len`.
    #[must_use]
    pub fn new(len: usize) -> Self {
        unsafe {
            let sexp = Rf_allocVector(VECSXP, len as R_xlen_t);
            List { sexp }
        }
    }
}

impl<T: HasSEXP> List<T> {
    fn get_sexp(&self) -> SEXP {
        self.sexp.get_sexp()
    }
}

impl<T: HasSEXP> HasSEXP for List<T> {
    fn get_sexp(&self) -> SEXP {
        self.sexp.get_sexp()
    }
}

impl<T: HasSEXP> TypedSEXP for List<T> {
    const SEXP_TYPE: SEXPTYPE = VECSXP;
}

unsafe impl<T: JustSEXP> JustSEXP for List<T> {
    type Inner = T;

    fn upcast(self) -> Self::Inner {
        self.sexp
    }

    fn inner_ref(&self) -> &Self::Inner {
        &self.sexp
    }

    fn wrap_sexp(sexp: SEXP) -> Option<Self> {
        if unsafe { TYPEOF(sexp) } == VECSXP {
            Some(List {
                sexp: T::wrap_sexp(sexp)?,
            })
        } else {
            None
        }
    }

    unsafe fn wrap_sexp_unchecked(sexp: SEXP) -> Self {
        List {
            sexp: T::wrap_sexp_unchecked(sexp),
        }
    }
}

impl<T: HasSEXP> IndexableSEXP for List<T> {
    type Index = usize;
    type Output = SEXP;

    fn len(&self) -> usize {
        unsafe { Rf_xlength(self.get_sexp()) as usize }
    }

    fn get_elt(&self, index: usize) -> Self::Output {
        self.check_inbound(index);
        unsafe { VECTOR_ELT(self.sexp.get_sexp(), index as R_xlen_t) }
    }

    fn set_elt(&mut self, index: usize, value: impl Into<Self::Output>) {
        self.check_inbound(index);
        unsafe {
            SET_VECTOR_ELT(self.sexp.get_sexp(), index as R_xlen_t, value.into());
        }
    }
}

impl<T: HasSEXP> DowncastSEXP<List<T>> for T {
    fn downcast(self) -> Option<List<T>> {
        if unsafe { TYPEOF(self.get_sexp()) } == VECSXP {
            Some(List { sexp: self })
        } else {
            None
        }
    }
}
