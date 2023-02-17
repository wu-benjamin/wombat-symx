# Wombat SymX

## Introduction

Wombat SymX is a symbolic executor that operates on LLVM IR (specifically, `*.bc` files) and uses a novel node-based approach.

## Setup

Note that LLVM 13+ is required for running the program and for creating `*.bc` LLVM binaries. This is packaged with the latest rust compiler (`rustc +1.60.0`).

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

## Results
./benchmark.sh
/usr/bin/time cargo run -- test_seq_br_safe_1.rs test

real    0m0.852s
user    0m0.166s
sys     0m0.113s
/usr/bin/time cargo run -- test_seq_br_safe_2.rs test

real    0m0.833s
user    0m0.173s
sys     0m0.115s
/usr/bin/time cargo run -- test_seq_br_safe_3.rs test

real    0m0.858s
user    0m0.179s
sys     0m0.129s
/usr/bin/time cargo run -- test_seq_br_safe_4.rs test

real    0m0.838s
user    0m0.176s
sys     0m0.115s
/usr/bin/time cargo run -- test_seq_br_safe_5.rs test

real    0m0.850s
user    0m0.181s
sys     0m0.119s
/usr/bin/time cargo run -- test_seq_br_safe_6.rs test

real    0m0.850s
user    0m0.185s
sys     0m0.118s
/usr/bin/time cargo run -- test_seq_br_safe_7.rs test

real    0m0.860s
user    0m0.190s
sys     0m0.121s
/usr/bin/time cargo run -- test_seq_br_safe_8.rs test

real    0m0.866s
user    0m0.198s
sys     0m0.125s
/usr/bin/time cargo run -- test_seq_br_safe_9.rs test

real    0m0.881s
user    0m0.206s
sys     0m0.125s
/usr/bin/time cargo run -- test_seq_br_safe_10.rs test

real    0m0.942s
user    0m0.216s
sys     0m0.121s
/usr/bin/time cargo run -- test_seq_br_safe_11.rs test

real    0m0.933s
user    0m0.214s
sys     0m0.116s
/usr/bin/time cargo run -- test_seq_br_safe_12.rs test

real    0m0.928s
user    0m0.232s
sys     0m0.117s
/usr/bin/time cargo run -- test_seq_br_safe_13.rs test

real    0m0.911s
user    0m0.240s
sys     0m0.120s
/usr/bin/time cargo run -- test_seq_br_safe_14.rs test

real    0m0.924s
user    0m0.266s
sys     0m0.121s
/usr/bin/time cargo run -- test_seq_br_safe_15.rs test

real    0m0.909s
user    0m0.253s
sys     0m0.123s
/usr/bin/time cargo run -- test_seq_br_safe_16.rs test

real    0m0.921s
user    0m0.268s
sys     0m0.118s
/usr/bin/time cargo run -- test_seq_br_safe_17.rs test

real    0m1.017s
user    0m0.365s
sys     0m0.116s
/usr/bin/time cargo run -- test_seq_br_safe_18.rs test

real    0m0.968s
user    0m0.304s
sys     0m0.120s
/usr/bin/time cargo run -- test_seq_br_safe_19.rs test

real    0m0.989s
user    0m0.321s
sys     0m0.120s
/usr/bin/time cargo run -- test_seq_br_safe_20.rs test

real    0m1.061s
user    0m0.395s
sys     0m0.125s
/usr/bin/time cargo run -- test_seq_br_unsafe_1.rs test

real    0m0.818s
user    0m0.167s
sys     0m0.114s
/usr/bin/time cargo run -- test_seq_br_unsafe_2.rs test

real    0m0.829s
user    0m0.174s
sys     0m0.117s
/usr/bin/time cargo run -- test_seq_br_unsafe_3.rs test

real    0m0.824s
user    0m0.176s
sys     0m0.117s
/usr/bin/time cargo run -- test_seq_br_unsafe_4.rs test

real    0m0.840s
user    0m0.184s
sys     0m0.119s
/usr/bin/time cargo run -- test_seq_br_unsafe_5.rs test

real    0m0.843s
user    0m0.189s
sys     0m0.123s
/usr/bin/time cargo run -- test_seq_br_unsafe_6.rs test

real    0m0.847s
user    0m0.192s
sys     0m0.120s
/usr/bin/time cargo run -- test_seq_br_unsafe_7.rs test

real    0m0.849s
user    0m0.199s
sys     0m0.120s
/usr/bin/time cargo run -- test_seq_br_unsafe_8.rs test

real    0m0.866s
user    0m0.209s
sys     0m0.118s
/usr/bin/time cargo run -- test_seq_br_unsafe_9.rs test

real    0m0.872s
user    0m0.220s
sys     0m0.121s
/usr/bin/time cargo run -- test_seq_br_unsafe_10.rs test

real    0m0.881s
user    0m0.225s
sys     0m0.118s
/usr/bin/time cargo run -- test_seq_br_unsafe_11.rs test

real    0m0.895s
user    0m0.238s
sys     0m0.127s
/usr/bin/time cargo run -- test_seq_br_unsafe_12.rs test

real    0m0.900s
user    0m0.250s
sys     0m0.116s
/usr/bin/time cargo run -- test_seq_br_unsafe_13.rs test

real    0m0.906s
user    0m0.253s
sys     0m0.120s
/usr/bin/time cargo run -- test_seq_br_unsafe_14.rs test

real    0m0.929s
user    0m0.279s
sys     0m0.115s
/usr/bin/time cargo run -- test_seq_br_unsafe_15.rs test

real    0m0.955s
user    0m0.300s
sys     0m0.118s
/usr/bin/time cargo run -- test_seq_br_unsafe_16.rs test

real    0m0.950s
user    0m0.297s
sys     0m0.118s
/usr/bin/time cargo run -- test_seq_br_unsafe_17.rs test

real    0m0.965s
user    0m0.316s
sys     0m0.115s
/usr/bin/time cargo run -- test_seq_br_unsafe_18.rs test

real    0m1.007s
user    0m0.359s
sys     0m0.115s
/usr/bin/time cargo run -- test_seq_br_unsafe_19.rs test

real    0m1.008s
user    0m0.362s
sys     0m0.115s
/usr/bin/time cargo run -- test_seq_br_unsafe_20.rs test

real    0m1.019s
user    0m0.368s
sys     0m0.115s
klee test_seq_br_safe_1.bc

real    0m0.019s
user    0m0.010s
sys     0m0.007s
klee test_seq_br_safe_2.bc

real    0m0.021s
user    0m0.010s
sys     0m0.009s
klee test_seq_br_safe_3.bc

real    0m0.021s
user    0m0.010s
sys     0m0.009s
klee test_seq_br_safe_4.bc

real    0m0.024s
user    0m0.011s
sys     0m0.010s
klee test_seq_br_safe_5.bc

real    0m0.027s
user    0m0.012s
sys     0m0.012s
klee test_seq_br_safe_6.bc

real    0m0.032s
user    0m0.014s
sys     0m0.015s
klee test_seq_br_safe_7.bc

real    0m0.041s
user    0m0.018s
sys     0m0.020s
klee test_seq_br_safe_8.bc

real    0m0.057s
user    0m0.027s
sys     0m0.028s
klee test_seq_br_safe_9.bc

real    0m0.096s
user    0m0.047s
sys     0m0.046s
klee test_seq_br_safe_10.bc

real    0m0.190s
user    0m0.094s
sys     0m0.090s
klee test_seq_br_safe_11.bc

real    0m0.366s
user    0m0.199s
sys     0m0.154s
klee test_seq_br_safe_12.bc

real    0m0.768s
user    0m0.436s
sys     0m0.306s
klee test_seq_br_safe_13.bc

real    0m1.799s
user    0m1.038s
sys     0m0.637s
klee test_seq_br_safe_14.bc

real    0m3.644s
user    0m2.194s
sys     0m1.247s
klee test_seq_br_safe_15.bc

real    0m7.829s
user    0m4.872s
sys     0m2.490s
klee test_seq_br_safe_16.bc

real    0m17.561s
user    0m10.799s
sys     0m5.118s
klee test_seq_br_safe_17.bc

real    0m39.714s
user    0m24.169s
sys     0m10.306s
klee test_seq_br_safe_18.bc

real    1m31.171s
user    0m53.965s
sys     0m21.447s
klee test_seq_br_safe_19.bc

real    2m33.179s
user    1m29.983s
sys     0m31.227s
klee test_seq_br_safe_20.bc

real    3m8.811s
user    1m55.259s
sys     0m36.060s
klee test_seq_br_unsafe_1.bc

real    0m0.037s
user    0m0.011s
sys     0m0.012s
klee test_seq_br_unsafe_2.bc

real    0m0.022s
user    0m0.010s
sys     0m0.010s
klee test_seq_br_unsafe_3.bc

real    0m0.023s
user    0m0.011s
sys     0m0.010s
klee test_seq_br_unsafe_4.bc

real    0m0.024s
user    0m0.011s
sys     0m0.011s
klee test_seq_br_unsafe_5.bc

real    0m0.027s
user    0m0.012s
sys     0m0.011s
klee test_seq_br_unsafe_6.bc

real    0m0.032s
user    0m0.014s
sys     0m0.015s
klee test_seq_br_unsafe_7.bc

real    0m0.035s
user    0m0.016s
sys     0m0.016s
klee test_seq_br_unsafe_8.bc

real    0m0.056s
user    0m0.025s
sys     0m0.027s
klee test_seq_br_unsafe_9.bc

real    0m0.068s
user    0m0.036s
sys     0m0.030s
klee test_seq_br_unsafe_10.bc

real    0m0.186s
user    0m0.078s
sys     0m0.066s
klee test_seq_br_unsafe_11.bc

real    0m0.219s
user    0m0.136s
sys     0m0.078s
klee test_seq_br_unsafe_12.bc

real    0m0.585s
user    0m0.360s
sys     0m0.216s
klee test_seq_br_unsafe_13.bc

real    0m1.029s
user    0m0.711s
sys     0m0.301s
klee test_seq_br_unsafe_14.bc

real    0m2.706s
user    0m1.743s
sys     0m0.811s
klee test_seq_br_unsafe_15.bc

real    0m4.565s
user    0m3.360s
sys     0m1.072s
klee test_seq_br_unsafe_16.bc

real    0m11.185s
user    0m8.113s
sys     0m2.728s
klee test_seq_br_unsafe_17.bc

real    0m22.088s
user    0m16.751s
sys     0m4.284s
klee test_seq_br_unsafe_18.bc

real    1m3.462s
user    0m42.744s
sys     0m13.712s
klee test_seq_br_unsafe_19.bc

real    1m28.542s
user    1m6.388s
sys     0m13.465s
klee test_seq_br_unsafe_20.bc

real    2m18.906s
user    1m38.201s
sys     0m23.540s