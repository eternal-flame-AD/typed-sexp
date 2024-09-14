# Typed-SEXP

Rust library for working with R's SEXP type in a type-safe way.

Build instructions:

## rasm

Inline assembler for R. 

```bash
cd crates/rasm

cargo build [--release]

# run demo
# please excuse the crudity of this model. 
# I didn't have time to build it to scale or paint it.
cd demo
Rscript --no-save demo.R
```

## Rust docs

```bash
cargo doc --open
```