use typed_sexp::{
    prelude::*,
    sexp::{
        env::{Env, Symbol},
        function::Callable,
        matrix::Matrix,
        vector::RealVector,
    },
};

#[export_name = "vector_add"]
pub extern "C" fn vector_add(a: SEXP, b: SEXP) -> SEXP {
    let a = a
        .coerce(SEXPTYPE::REALSXP)
        .downcast_to::<RealVector<_>>()
        .expect_r("a is not a numeric vector")
        .protect();
    let b = b
        .coerce(SEXPTYPE::REALSXP)
        .downcast_to::<RealVector<_>>()
        .expect_r("b is not a numeric vector")
        .protect();

    let out_len = a.len().min(b.len());

    let mut out = RealVector::new(out_len).protect();

    for i in 0..out_len {
        out[i] = a[i] + b[i];
    }

    out.get_sexp()
}

#[export_name = "matrix_multiply"]
pub extern "C" fn matrix_multiply(mat: SEXP, vec: SEXP) -> SEXP {
    let mat = mat
        .coerce(SEXPTYPE::REALSXP)
        .downcast_to::<RealVector<_>>()
        .expect_r("mat is not a numeric vector")
        .downcast_to::<Matrix<_>>()
        .expect_r("mat is not a matrix")
        .protect();
    let vec = vec
        .coerce(SEXPTYPE::REALSXP)
        .downcast_to::<RealVector<_>>()
        .expect_r("vec is not a numeric vector")
        .protect();

    let mat_rows = mat.nrows();
    let mat_cols = mat.ncols();

    if mat_cols != vec.len() {
        Err::<(), _>("mat_cols != vec.len()").unwrap_r();
    }

    let mut out = RealVector::new(mat_rows).protect();

    for i in 0..mat_rows {
        let mut sum = 0.0;
        for j in 0..mat_cols {
            sum += mat[(i, j)] * vec[j];
        }
        out[i] = sum;
    }

    out.get_sexp()
}

/// A .Call() function that takes an R callable and calls it with two arguments.
#[export_name = "call_back"]
pub extern "C" fn call_back(cb: SEXP, number_arg: SEXP) -> SEXP {
    let cb = cb
        .downcast_to::<Callable<_>>()
        .expect_r("cb is not a callable")
        .protect();

    cb.build_pairlist()
        .push(IntegerVectorSEXP::scalar(1).protect())
        .push(CharacterVectorSEXP::scalar("Hello, world!").protect())
        .push_tagged(
            Symbol::new("number").protect(),
            number_arg.coerce(SEXPTYPE::REALSXP).protect(),
        )
        .build_lang()
        .eval(Env::current())
        .unwrap_r()
}
