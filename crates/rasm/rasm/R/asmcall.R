#' Asemble a string of assembly code
#'
#' Currently only NASM is supported.
#'
#' Supported sections;
#'   - .data
#'   - .rodata
#'   - .text
#'   - .rela.text
#'   - .rel.text
#'
#' Runtime linked APIs such as R APIs can be declared as `extern`.
#'
#' @param asm A scalar string of assembly code
#' @return A handle to the linked assembly code
#'
#' @export
assemble <- function(asm, flavor = "nasm") {
    if (flavor != "nasm") {
        stop("Only NASM is supported at the moment!")
    }
    .Call("assemble", asm)
}

#' Call an assembly function in the .C calling convention
#'
#' The underlying code are expected to accept SEXP's and return one SEXP.
#'
#' @param box A handle to the linked assembly code
#' @param name The name of the function to call
#' @param ... Arguments to pass to the assembly function
#'
#' @return The return value of the assembly function
#'
#' @export
.Asm <- function(box, name, ...) {
    .Call("asm_call", box, name, list(...))
}
