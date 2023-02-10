use std::fs;
use std::process::Command;

use tracing_core::Level;
use tracing_subscriber::FmtSubscriber;

use wombat_symx;

pub fn test(test_name: &str, function_name: &str, source_code: &str, expected_safe: bool) -> () {

    // Setup the tracing debug level
    let subscriber = FmtSubscriber::builder().with_max_level(Level::DEBUG).finish();

    // _guard resets the current default dispatcher to the prior default when dropped
    let _guard = tracing::subscriber::set_default(subscriber);

    let source_file_name = format!("tests/zzz_temp_test_{}.rs", test_name);
    let bytecode_file_name = format!("tests/zzz_temp_test_{}.bc", test_name);

    // Prevent compiler from optimizing away unused function
    let main = format!("fn main() {{println!(\"{{:p}}\", {} as *const ())}}", function_name);

    fs::write(&source_file_name, format!("{}\n{}", source_code, main)).expect("Failed to write temp test file!");
    Command::new("rustc")
        .args(["--emit=llvm-bc", &source_file_name, "-o", &bytecode_file_name])
        .status()
        .expect("Failed to generate bytecode file!");

    let actual_safe = wombat_symx::symbolic_execution::symbolic_execution(&bytecode_file_name, &String::from(function_name));

    fs::remove_file(&source_file_name).expect("Failed to delete temp test source file.");
    fs::remove_file(&bytecode_file_name).expect("Failed to delete temp test bytecode file.");

    assert!(expected_safe == actual_safe.unwrap());
}