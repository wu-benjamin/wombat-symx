use std::fs;
use std::process::Command;

use tracing_core::Level;
use tracing_subscriber::FmtSubscriber;

struct TestFileDropper<'a> {
    source_file_name: &'a String,
    bytecode_file_name: &'a String,
}

impl Drop for TestFileDropper<'_> {
    fn drop(&mut self) {
        fs::remove_file(self.source_file_name).expect("Failed to delete temp test source file.");
        fs::remove_file(self.bytecode_file_name).expect("Failed to delete temp test bytecode file.");
    }
}

pub fn test(test_name: &str, function_name: &str, source_code: &str, expected_safe: bool) -> () {
    let debug = false;

    // Setup the tracing debug level
    let subscriber = if debug {
        FmtSubscriber::builder().with_max_level(Level::DEBUG).finish()
    } else {
        FmtSubscriber::builder().with_max_level(Level::WARN).finish()
    };

    // _guard resets the current default dispatcher to the prior default when dropped
    let _guard = tracing::subscriber::set_default(subscriber);

    let source_file_name = format!("tests/zzz_temp_test_{}.rs", test_name);
    let bytecode_file_name = format!("tests/zzz_temp_test_{}.bc", test_name);

    let _test_file_dropper = TestFileDropper {
        source_file_name: &source_file_name,
        bytecode_file_name: &bytecode_file_name,
    };

    // Prevent compiler from optimizing away unused function
    let main = format!("fn main() {{println!(\"{{:p}}\", {} as *const ())}}", function_name);

    fs::write(&source_file_name, format!("{}\n{}", source_code, main)).expect("Failed to write temp test file!");
    Command::new("rustc")
        .args(["--emit=llvm-bc", &source_file_name, "-o", &bytecode_file_name])
        .status()
        .expect("Failed to generate bytecode file!");

    let actual_safe = wombat_symx::symbolic_execution::symbolic_execution(&bytecode_file_name, &String::from(function_name));

    assert!(expected_safe == actual_safe.unwrap());
}