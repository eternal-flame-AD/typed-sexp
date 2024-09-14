use typed_sexp::{
    embedded::EmbeddedR,
    prelude::*,
    sexp::{
        env::{Env, Symbol},
        function::Builtin,
        vector::CharacterVector,
    },
};

const CODE: &str = r#"
library(purrr)
library(dplyr)

or <- function(x, y) {
    if (length(x) == 0) {
        return(y)
    }
    x
}

find_common_order <- function(input) {
    dots <- map(input, unique)
    elements <- unique(unlist(dots), use.names = FALSE)
    out <- vector(mode(elements), length(elements))

    for (i in seq_along(elements)) {
        first_elements <- dots |> compact() |> map_vec(1, .ptype = elements)

        max_positions <- first_elements |> map_int(
            function(e) max(map_int(dots, ~ max(which(. == e) %or% 1L)))
        )

        out[i] <- first_elements[max_positions == min(max_positions)][1]

        dots <- dots |> map(setdoff, out[i])
    }
    out
}
"#;

fn main() {
    let _embed = unsafe { EmbeddedR::init() };

    let code_sexp = CharacterVector::scalar(CODE).protect();

    Env::base()
        .peek(Symbol::new("eval"))
        .unwrap_r()
        .downcast_to::<Builtin<_>>()
        .unwrap_r()
        .protect()
        .build_pairlist()
        .push(code_sexp)
        .build_lang()
        .eval(Env::global())
        .unwrap_r();
}
