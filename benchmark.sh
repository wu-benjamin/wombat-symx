for n in {1..20}
do
    python3 generate_test_seq_br.py rust $n safe
    echo /usr/bin/time cargo run -- test_seq_br_safe_$n.rs test
    time ( cargo run -- test_seq_br_safe_$n.rs test > /dev/null 2>&1 ) 2>&1
done

for n in {1..20}
do
    python3 generate_test_seq_br.py rust $n unsafe
    echo /usr/bin/time cargo run -- test_seq_br_unsafe_$n.rs test
    time ( cargo run -- test_seq_br_unsafe_$n.rs test > /dev/null 2>&1 ) 2>&1
done

for n in {1..20}
do
    python3 generate_test_seq_br.py c $n safe
    clang -I "/opt/homebrew/Cellar/klee/2.3_4/include/klee" -emit-llvm -c -g -O0 -Xclang -disable-O0-optnone test_seq_br_safe_$n.c
    echo klee test_seq_br_safe_$n.bc
    time ( klee test_seq_br_safe_$n.bc > /dev/null 2>&1 ) 2>&1
done

for n in {1..20}
do
    python3 generate_test_seq_br.py c $n unsafe
    clang -I "/opt/homebrew/Cellar/klee/2.3_4/include/klee" -emit-llvm -c -g -O0 -Xclang -disable-O0-optnone test_seq_br_unsafe_$n.c
    echo klee test_seq_br_unsafe_$n.bc
    time ( klee test_seq_br_unsafe_$n.bc > /dev/null 2>&1 ) 2>&1
done