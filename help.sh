cd tests
for filename in ./*.rs; do
  rustc --emit=llvm-ir $filename
  rustc --emit=llvm-bc $filename
done
