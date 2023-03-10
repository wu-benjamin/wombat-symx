// Copyright (c) 2023 Benjamin Jialong Wu
// This code is licensed under MIT license (see LICENSE.md for details)

use std::fs;
use std::path::Path;

use tracing_core::Level;
use tracing_subscriber::FmtSubscriber;

struct FileDropper<'a> {
    file_name: &'a String,
}

impl Drop for FileDropper<'_> {
    fn drop(&mut self) {
        fs::remove_file(self.file_name).expect("Failed to delete file.");
    }
}

pub fn test(test_name: &str, function_name: &str, source_code: &str, expected_safe: bool) {
    // Needs to be manually changed due to the test harness not handling cmd args
    let debug = false;

    // Setup the tracing debug level
    let subscriber = if debug {
        FmtSubscriber::builder().with_max_level(Level::DEBUG).finish()
    } else {
        FmtSubscriber::builder().with_max_level(Level::WARN).finish()
    };

    // _guard resets the current default dispatcher to the prior default when dropped
    let _guard = tracing::subscriber::set_default(subscriber);

    let source_file_name = format!("tests_temp/zzz_temp_test_{}.rs", test_name);
    if !Path::exists(Path::new("tests_temp")) {
        // Ensure temp test directory exists, otherwise src file fails to be created
        fs::create_dir("tests_temp").expect("Unable to create test directory: \"tests_temp\"");
    }

    let _file_dropper = FileDropper { file_name: &source_file_name };

    // Prevent compiler from optimizing away unused function
    let main = format!("fn main() {{println!(\"{{:p}}\", {} as *const ())}}", function_name);

    fs::write(&source_file_name, format!("{}\n{}", source_code.replace("            ", ""), main)).expect("Failed to write temp test file!");

    let actual_safe = wombat_symx::symbolic_execution::symbolic_execution(&source_file_name, &String::from(function_name));

    assert!(expected_safe == actual_safe.unwrap());
}
