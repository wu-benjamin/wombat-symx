ITERATIONS=20

cargo build --release

# Used for klee output as many folders are created normally
tmpkleedir=$(mktemp -d 2>/dev/null)

# Stores the hyperfine output for timings
mkdir benchmark_output

echo "Running benchmark on safe functions with Wombat-SymX"
for n in $(seq $ITERATIONS)
do
    python3 generate_test_seq_br.py rust $n safe
    rustc --emit=llvm-bc test_seq_br_safe_$n.rs
    echo /usr/bin/time cargo run -- test_seq_br_safe_$n.rs test
    hyperfine --warmup 2 --export-csv benchmark_output/wombat_safe_$n.csv "target/release/wombat_symx -b test_seq_br_safe_$n.rs test > /dev/null 2>&1"
    rm test_seq_br_safe_$n.*
done

echo
echo
echo "Running benchmark on unsafe functions with Wombat-SymX"
for n in $(seq $ITERATIONS)
do
    python3 generate_test_seq_br.py rust $n unsafe
    rustc --emit=llvm-bc test_seq_br_unsafe_$n.rs
    echo /usr/bin/time cargo run -- test_seq_br_unsafe_$n.rs test
    hyperfine --warmup 2 --export-csv benchmark_output/wombat_unsafe_$n.csv "target/release/wombat_symx -b test_seq_br_unsafe_$n.rs test > /dev/null 2>&1"
    rm test_seq_br_unsafe_$n.*
done

echo
echo
echo "Running benchmark on safe functions with KLEE"
for n in $(seq $ITERATIONS)
do
    python3 generate_test_seq_br.py c $n safe
    clang -I "/opt/homebrew/Cellar/klee/2.3_4/include/klee" -emit-llvm -c -g -O0 -Xclang -disable-O0-optnone test_seq_br_safe_$n.c
    echo klee test_seq_br_safe_$n.bc
    hyperfine --warmup 2 --prepare "rm -rf $tmpkleedir/klee_safe_$n" --cleanup "rm -rf $tmpkleedir/klee_safe_$n" --export-csv benchmark_output/klee_safe_$n.csv "klee --output-dir $tmpkleedir/klee_safe_$n test_seq_br_safe_$n.bc > /dev/null 2>&1"
    rm test_seq_br_safe_$n.*
done

echo
echo
echo "Running benchmark on unsafe functions with KLEE"
for n in $(seq $ITERATIONS)
do
    python3 generate_test_seq_br.py c $n unsafe
    clang -I "/opt/homebrew/Cellar/klee/2.3_4/include/klee" -emit-llvm -c -g -O0 -Xclang -disable-O0-optnone test_seq_br_unsafe_$n.c
    echo klee test_seq_br_unsafe_$n.bc
    hyperfine --warmup 2 --prepare "rm -rf $tmpkleedir/klee_unsafe_$n" --cleanup "rm -rf $tmpkleedir/klee_unsafe_$n" --export-csv benchmark_output/klee_unsafe_$n.csv "klee --output-dir $tmpkleedir/klee_unsafe_$n test_seq_br_unsafe_$n.bc > /dev/null 2>&1"
    rm test_seq_br_unsafe_$n.*
done

echo "Performing cleanup of extra files:"
rm -rvf $tmpkleedir
