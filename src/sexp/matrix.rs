use std::{
    fmt::{Debug, Display},
    ops::{Index, IndexMut},
};

use derive_more::Deref;
use libR_sys::SEXP;

use crate::{DowncastSEXP, HasSEXP, IndexableSEXP, JustSEXP, TypedSEXP};

#[repr(transparent)]
#[derive(Deref)]
/// A wrapper around a matrix of a given type.
pub struct Matrix<U: JustSEXP + TypedSEXP + IndexableSEXP> {
    matrix: U,
}

impl<U: JustSEXP + TypedSEXP + IndexableSEXP> Debug for Matrix<U> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Matrix ({:?}, {}x{})",
            U::SEXP_TYPE,
            self.nrows(),
            self.ncols()
        )
    }
}

impl<U: JustSEXP + TypedSEXP + IndexableSEXP> Display for Matrix<U>
where
    Self: IndexableSEXP<Index = (usize, usize), Output = U::Output>,
    <U as IndexableSEXP>::Output: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Matrix [")?;

        for i in 0..self.nrows() {
            write!(f, "[")?;
            for j in 0..self.ncols() {
                write!(f, "{}", self.get_elt((i, j)))?;
                if j != self.ncols() - 1 {
                    write!(f, ", ")?;
                }
            }
            write!(f, "]")?;
            if i != self.nrows() - 1 {
                write!(f, ", ")?;
            }
        }

        write!(f, "]")
    }
}

impl<U: JustSEXP + TypedSEXP + IndexableSEXP> Matrix<U> {
    #[must_use]
    /// Create a new matrix with the given number of rows and columns.
    pub fn new(nrow: usize, ncol: usize) -> Self
    where
        U: JustSEXP<Inner = SEXP>,
    {
        unsafe {
            let sexp = libR_sys::Rf_allocMatrix(U::SEXP_TYPE, nrow as _, ncol as _);
            Matrix {
                matrix: U::wrap_sexp_unchecked(sexp),
            }
        }
    }

    #[must_use]
    /// The number of rows in the matrix.
    pub fn nrows(&self) -> usize {
        unsafe { libR_sys::Rf_nrows(self.matrix.get_sexp()) as usize }
    }

    #[must_use]
    /// The number of columns in the matrix.
    pub fn ncols(&self) -> usize {
        unsafe { libR_sys::Rf_ncols(self.matrix.get_sexp()) as usize }
    }
}

impl<U> HasSEXP for Matrix<U>
where
    U: JustSEXP + TypedSEXP + IndexableSEXP<Index = usize>,
{
    fn get_sexp(&self) -> libR_sys::SEXP {
        self.matrix.get_sexp()
    }
}

unsafe impl<U> JustSEXP for Matrix<U>
where
    U: JustSEXP + TypedSEXP + IndexableSEXP<Index = usize>,
{
    type Inner = U::Inner;

    fn upcast(self) -> Self::Inner {
        self.matrix.upcast()
    }

    fn inner_ref(&self) -> &Self::Inner {
        self.matrix.inner_ref()
    }

    fn wrap_sexp(sexp: SEXP) -> Option<Self> {
        if unsafe { libR_sys::Rf_isMatrix(sexp) }.into() {
            U::wrap_sexp(sexp).map(|inner| Matrix { matrix: inner })
        } else {
            None
        }
    }

    unsafe fn wrap_sexp_unchecked(sexp: SEXP) -> Self {
        Matrix {
            matrix: U::wrap_sexp_unchecked(sexp),
        }
    }
}

impl<U> IndexableSEXP for Matrix<U>
where
    U: JustSEXP + TypedSEXP + IndexableSEXP<Index = usize>,
{
    type Index = (usize, usize);
    type Output = U::Output;

    fn len(&self) -> usize {
        self.nrows() * self.ncols()
    }

    fn get_elt(&self, index: Self::Index) -> Self::Output {
        let (row, col) = index;
        self.matrix.get_elt(row * self.ncols() + col)
    }

    fn set_elt(&mut self, index: Self::Index, value: impl Into<Self::Output>) {
        let (row, col) = index;
        self.matrix.set_elt(row * self.ncols() + col, value);
    }
}

impl<U> Index<(usize, usize)> for Matrix<U>
where
    U: JustSEXP + TypedSEXP + IndexableSEXP<Index = usize> + Index<usize>,
{
    type Output = <U as Index<usize>>::Output;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        &self.matrix[index.0 * self.ncols() + index.1]
    }
}

impl<U> IndexMut<(usize, usize)> for Matrix<U>
where
    U: JustSEXP + TypedSEXP + IndexableSEXP<Index = usize> + IndexMut<usize>,
{
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        let ncols = self.ncols();
        &mut self.matrix[index.0 * ncols + index.1]
    }
}

impl<U> DowncastSEXP<Matrix<U>> for U
where
    U: JustSEXP + TypedSEXP + IndexableSEXP<Index = usize>,
{
    fn downcast(self) -> Option<Matrix<U>> {
        if unsafe { libR_sys::Rf_isMatrix(self.get_sexp()) }.into() {
            Some(Matrix { matrix: self })
        } else {
            None
        }
    }
}
