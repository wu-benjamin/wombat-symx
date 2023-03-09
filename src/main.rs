// Copyright (c) 2023 Benjamin Jialong Wu
// This code is licensed under MIT license (see LICENSE.md for details)

use clap::Parser;

use tracing_core::Level;
use tracing_subscriber::FmtSubscriber;

use wombat_symx::symbolic_execution::symbolic_execution;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Enable debug printing
    #[clap(short, long)]
    debug: bool,

    /// Set file name to perform symbolic execution on
    #[clap()]
    file_name: String,

    /// Set function to perform symbolic execution on
    #[clap()]
    function_name: String,
}

fn main() {
    let features = Args::parse();

    // Setup the tracing debug level
    let subscriber = if features.debug {
        FmtSubscriber::builder().with_max_level(Level::DEBUG).finish()
    } else {
        FmtSubscriber::builder().with_max_level(Level::WARN).finish()
    };
    // _guard resets the current default dispatcher to the prior default when dropped
    let _guard = tracing::subscriber::set_default(subscriber);

    let file_name = String::from(&features.file_name);
    let function_name = String::from(&features.function_name);

    symbolic_execution(&file_name, &function_name);
}
