#' Rasm: Inline Assembler for R
#'
#' This package provides a way to write inline assembly code in R.
#'
#' It is mostly a proof of concept and should not be used in production.
#'
#' However it has the following "advantages" to traditional ways to write R extensions (mostly joking):
#'
#' 1. CRAN friendly: CRAN disallow "binary executable code" in packages, but
#'    does not disallow unlinked assembly code that links itself at runtime.
#' 2. No need to get a working compiler with R headers,
#'    Just write the name of the function you need and this package will call it for you!
#' 3. Hot reloading: Replace your native code without restarting R! Just delete the old page and create a new one.
#' 4. Utmost compatibility: If R runs on this machine, your code will 100% run too!
#' 5. Unbeatable control: You can do anything you want, even if it's a bad idea.
#'
#' @docType package
#' @name rasm
#' @useDynLib rasm
"_PACKAGE"

.onLoad <- function(libname, pkgname) {
    if (Sys.getenv("RASM_LOG") == "1") {
        .Call("init_rustlog")
    }
}
