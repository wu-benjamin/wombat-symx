cargo build
for filefullpath in tests/*.bc; do
  filename="$(basename -- $filefullpath)"
  cargo run -- -d $filefullpath > tests/output/"${filename%.*}".txt
done
