dyn.load("../../../target/debug/librasm.so")

invisible(.Call("init_rustlog"))

assemble <- function(asm, flavor = "nasm") {
    if (flavor != "nasm") {
        stop("Only NASM is supported at the moment!")
    }
    .Call("assemble", asm)
}

.Asm <- function(box, name, ...) {
    .Call("asm_call", box, name, list(...))
}

code <- file("sabotageR.s", "r")

asm <- assemble(paste(readLines(code), collapse = "\n"))

y = 0 # I don't like this!!! Do something about it!

invisible(.Asm(asm, "sabotage", "="))

x <- 1
print(sprintf("`<-` still works! x is now: %d", x))
# [1] "`<-` still works! x is now: 1"

tryCatch({
    y = 2
}, error = function(e) {
    print(e) # <simpleError in y = 2: This is R, use <- instead of `=` :D
}, finally = {
    print(sprintf("y is still: %d", y)) # y is still: 0
})
