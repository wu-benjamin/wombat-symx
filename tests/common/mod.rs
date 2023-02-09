use std::fs;
use std::process::Command;
use std::env::current_dir;

use wombat_symx;

pub fn test(test_name: &str, function_name: &str, source_code: &str, expected_safe: bool) -> () {
    let source_file_name = format!("tests/ZZZ_temp_test_{}.rs", test_name);
    let bytecode_file_name = format!("ZZZ_temp_test_{}.bc", test_name);
    let current_directory_pathbuf = current_dir().unwrap();
    let current_directory_path = current_directory_pathbuf.as_path();
    let canonical_current_directory = fs::canonicalize(current_directory_path);
    let canonical_current_directory_pathbuf = canonical_current_directory.unwrap();
    let canonical_current_directory_path = canonical_current_directory_pathbuf.as_path();

    // Prevent compiler from optimizing away unused function
    let main = format!("fn main() {{println!(\"{{:p}}\", {} as *const ())}}", function_name);

    fs::write(&source_file_name, format!("{}\n{}", source_code, main)).expect("Failed to write temp test file!");
    Command::new("rustc")
        .args(["--emit=llvm-bc", &source_file_name])
        .current_dir(canonical_current_directory_path)
        .status()
        .expect("Failed to generate bytecode file!");

    let actual_safe = wombat_symx::symbolic_execution::symbolic_execution(&bytecode_file_name, &String::from(function_name));

    fs::remove_file(&source_file_name).expect("Failed to delete temp test source file.");
    fs::remove_file(&bytecode_file_name).expect("Failed to delete temp test bytecode file.");

    assert!(expected_safe == actual_safe.unwrap());
}