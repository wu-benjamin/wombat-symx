ITERATIONS=20


cp benchmark_output/wombat_safe_1.csv benchmark_output/combined_wombat_safe.csv
cp benchmark_output/wombat_unsafe_1.csv benchmark_output/combined_wombat_unsafe.csv
cp benchmark_output/klee_safe_1.csv benchmark_output/combined_klee_safe.csv
cp benchmark_output/klee_unsafe_1.csv benchmark_output/combined_klee_unsafe.csv

for n in $(seq 2 $ITERATIONS)
do
    cat benchmark_output/wombat_safe_$n.csv | tail -1 >> benchmark_output/combined_wombat_safe.csv
    cat benchmark_output/wombat_unsafe_$n.csv | tail -1 >> benchmark_output/combined_wombat_unsafe.csv
    cat benchmark_output/klee_safe_$n.csv | tail -1 >> benchmark_output/combined_klee_safe.csv
    cat benchmark_output/klee_unsafe_$n.csv | tail -1 >> benchmark_output/combined_klee_unsafe.csv
done
