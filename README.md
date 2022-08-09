# Wombat SymX

## Introduction

Wombat SymX is a symbolic executor that operates on LLVM IR (specifically, `*.bc` files) and uses a novel node-based approach.

## Setup

Note that LLVM 13+ is required for running the program and for creating `*.bc` LLVM binaries. This is packaged with the latest rust compiler (`rustc +1.60.0`)

For MAC and Linux, run:
```zsh
curl https://sh.rustup.rs -sSf | sh
```

### Mac Dependency Installation

You will need the following dependencies:
- CMake (`brew install cmake`)
- Swig (`brew install swig`)
- LLVM (`brew install llvm`)

Run `brew info llvm` to see information from the llvm installation. Run the command to add llvm to the `$PATH`.

Lastly, add the prefix for llvm (as seen in `brew info llvm`) to the environment variable `LLVM_SYS_130_PREFIX`.
- ex: `export LLVM_SYS_130_PREFIX="/opt/homebrew/opt/llvm"`

### Build Project

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

## Creating LLVM IR files

To create `bc` files containing LLVM IR that Wombat SymX can use, run the following command:
```zsh
rustc --emit=llvm-bc <file-name>.rs
```

A human-readable LLVM IR format can be created by using the following:
```zsh
rustc --emit=llvm-ir <file-name>.rs
```

## Create LLVM IR from Test Files

To compile all of the test rust files into LLVM bytecode files, run the following help script in the project root:
```zsh
./help.sh
```
