# Typed-SEXP

Type-safe manipulation of R's SEXP objects without bells and whistles.

## Objectives

R internal API is very confusing and not type-safe.

However many times I do not want to pull in huge proc macros just to write my R extension.
The main reasons are:
  - Macros in general are hard to debug.
  - Macros mess up static analysis tools.
  - Macros hide the actual code that is being generated.

This library is an attempt to provide a type-safe interface to R's SEXP objects without any user-exposed macros. 
However it is not a goal to provide a high-level abstractions over R's API like `Rcpp` or `extendr`, the most I will do is type-safe indexing, attributes, calling functions, etc.
Users are expected to have a general understanding of R internals. 
Here is a quick [tutorial](https://github.com/hadley/r-internals).

## Features

- Typed SEXP objects that are ABI-compatible with R's SEXP.
- Type-safe (mutable) indexing of vectors and matrices.
- Typed protection of SEXP objects with a debug mode that sanity-checks the protection order.
- Stack RAII-based auto un-protection of SEXP objects.
- Downcasting of SEXP objects in a method chain with a single call (even if the object is wrapped in another abstraction).
- Namespace and function objects that are environment-aware.
- Call R functions with type-safe arguments and return values.
- (TODO): Dynamically create R functions.

## Rust docs

Hosted on [docs.rs](https://docs.rs/typed-sexp)

Or build locally with:

```bash
cargo doc --open
```

## Examples

### extension-demo

Located in [`crates/extension-demo`](crates/extension-demo)

A demo on writing R-extension with this, include a simple vector addition, matrix multiplication, and calling back a closure passed from R with additional arguments.

### embedded-demo

Located in [`crates/embedded-demo`](crates/embedded-demo)

A demo on embedding R in Rust, and using R from a pure Rust program.

### rasm

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

