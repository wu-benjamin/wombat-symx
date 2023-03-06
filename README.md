# Wombat SymX

## Introduction

Wombat SymX is a symbolic executor that operates on LLVM IR (specifically, `*.bc` files) and uses a novel node-based approach.

## Setup

Note that LLVM 13 is required for running the program and for creating `*.bc` LLVM binaries. This is packaged with the following rust compilers (`rustc 1.60.*-1.64.*`).

For MAC and Linux, run:
```zsh
curl https://sh.rustup.rs -sSf | sh
```

Project is tested on an M1 Pro Macbook 2021 with `rustc 1.60.0 (7737e0b5c 2022-04-04)` and `llvm-13.0.1`.

### Mac Dependency Installation

You will need the following dependencies:
- CMake (`brew install cmake`)
- Swig (`brew install swig`)
- LLVM (`brew install llvm@13`)

Run `brew info llvm@13` to see information from the llvm installation. Run the command to add llvm to the `$PATH`.

Lastly, add the prefix for llvm (as seen in the installation path from `brew info llvm@13`) to the environment variable `LLVM_SYS_130_PREFIX`.
- ex: `export LLVM_SYS_130_PREFIX="/opt/homebrew/opt/llvm@13"`

### Build Project

To build the project, use:
```
cargo build
```

### Update Project Dependencies

To updat the project dependencies, use:
```
cargo update
```
Note it is wise to backup your local dependencies incase an external dependency is updated in a breaking way and is not properly versioned.

## Runtime Execution

To see an overview of run commands, use the following:
```bash
cargo run -- --help
```

To run the project, use:
```
cargo run -- [rs-file-path] [function-name]
```

To run the project with debug output enabled, use:
```
cargo run -- -d [rs-file-path] [function-name]
```

## Run Test Suite

To run all (integration) test case functions (optionally matching a prefix), use:
```
cargo test [test-prefix]
```

To run test case functions with output, use:
```
cargo test [test-prefix] -- --show-output
```


## Creating LLVM IR files

To create `bc` files containing LLVM IR that Wombat SymX can use, run the following command:
```zsh
rustc --emit=llvm-bc <file-name>.rs
```

A human-readable LLVM IR format can be created by using the following:
```zsh
rustc --emit=llvm-ir <file-name>.rs
```

# Benchmarking
## Install KLEE

`brew install klee`

## Add to Include Path

Add the path to the directory (check with `brew info klee`) with the header file to the C include path.
The path looks like the following:
`"/opt/homebrew/Cellar/klee/2.3_4/include/klee"`

## Generate Test Cases
`python3 generate_test_seq_br.py <language> <number_of_branches> <safety>`

## Compile C Code for KLEE
`clang -I "/opt/homebrew/Cellar/klee/2.3_4/include/klee" -emit-llvm -c -g -O0 -Xclang -disable-O0-optnone <c_file>`

## Time KLEE
`time klee <c_bc_file>`

## Time Wombat SymX
`time cargo run -- <rust_file> test`  


# Resources

## LLVM Unsigned vs Signed

LLVM lifts all integers to signed. Intrinsic functions still use unsigned operations while taking signed integers as arguments.

https://stackoverflow.com/questions/14723532/llvms-integer-types
