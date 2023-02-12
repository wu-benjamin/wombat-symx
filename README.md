# Wombat SymX

## Introduction

Wombat SymX is a symbolic executor that operates on LLVM IR (specifically, `*.bc` files) and uses a novel node-based approach.

## Setup

Note that LLVM 13+ is required for running the program and for creating `*.bc` LLVM binaries. This is packaged with the latest rust compiler (`rustc +1.60.0`).

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

### Update Project Dependencies

To build the project, use:
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
cargo run -- [bc-file-path] [function-name]
```

To run the project with debug output enabled, use:
```
cargo run -- -d [bc-file-path] [function-name]
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
