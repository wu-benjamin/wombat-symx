# Wombat SymX

## Introduction

Wombat SymX is a symbolic executor that operates on LLVM IR (specifically, `*.bc` files) and uses a novel node-based approach.

## Setup

Note that LLVM 13+ is required for running the program and for creating `*.bc` LLVM binaries. This is packaged with the latest rust compiler (`rustc +1.60.0`)

To build the project, use:
```
cargo build
```

## Runtime Execution

To run the project, use:
```
cargo run [bc-file-path]
```

If no `bc-file-path` is provided, then the program will default to `tests/hello_world.rs` under the `neg_abs` function.