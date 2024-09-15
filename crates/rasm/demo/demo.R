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

# Above is supposed to be part of the package, below is the demo code

wait <- function(pid) {
    exit_labels <- c("exited", "killed", "dumped", "trapped", "stopped", "continued")
    print(sprintf("I am the parent R process! My child is: %d", pid))
    status <- .Asm(asm, "waitpidr", pid) # Dummy, an exercise for the reader
    print(sprintf("My child exited with status: %s!", exit_labels[status]))
}

code <- file("forkR.s", "r")

asm <- assemble(paste(readLines(code), collapse = "\n"))

pid <- .Asm(asm, "forkr")

if (pid == 0) {
    print(sprintf("I am the child R process! My PID is: %d", Sys.getpid()))
    print("Crashing!")
    .Asm(asm, "crashpls")
}

wait(pid)

pid <- .Asm(asm, "forkr")

if (pid == 0) {
    print("I am the child R process, I am exiting normally this time!")
    quit()
}

wait(pid)
