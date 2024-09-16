# Typed-SEXP

Rust library for working with R's SEXP type in a type-safe way.

Build instructions:

## rasm

Joke project and also serves as a testbed for the stability of this library.

Inline assembler for R. 

[Blog Post](https://yumechi.jp/en/blog/2024/dynamically-load-assembler-code-in-r/)

```bash
cd crates/rasm

cargo build [--release]

# run demo
# please excuse the crudity of this model. 
# I didn't have time to build it to scale or paint it.
cd demo
Rscript --no-save fork.R
Rscript --no-save sabotage.R
```

## Rust docs

```bash
cargo doc --open
```