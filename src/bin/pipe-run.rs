//! CLI tool to run pipeline (.pipe) files against input data.
//!
//! Usage:
//!   pipe-run <pipeline.pipe> <input.data>
//!   pipe-run <pipeline.pipe> <input.data> -o <output.data>
//!
//! If no output file is specified, writes to stdout.

use pipelines_rs::execute_pipeline;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        eprintln!(
            "Usage: {} <pipeline.pipe> <input.data> [-o output.data]",
            args[0]
        );
        eprintln!();
        eprintln!("Run a pipeline file against input data.");
        eprintln!();
        eprintln!("Arguments:");
        eprintln!("  <pipeline.pipe>  Pipeline definition file (.pipe)");
        eprintln!("  <input.data>     Input data file (80-byte records)");
        eprintln!("  -o <output>      Optional output file (default: stdout)");
        process::exit(1);
    }

    let pipe_file = &args[1];
    let input_file = &args[2];
    let output_file = if args.len() > 4 && args[3] == "-o" {
        Some(&args[4])
    } else {
        None
    };

    // Read pipeline file
    let pipeline_text = match fs::read_to_string(pipe_file) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading pipeline file '{}': {}", pipe_file, e);
            process::exit(1);
        }
    };

    // Read input file
    let input_text = match fs::read_to_string(input_file) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading input file '{}': {}", input_file, e);
            process::exit(1);
        }
    };

    // Execute pipeline
    match execute_pipeline(&input_text, &pipeline_text) {
        Ok((output, input_count, output_count)) => {
            // Write output
            if let Some(out_path) = output_file {
                if let Some(parent) = Path::new(out_path).parent()
                    && !parent.as_os_str().is_empty()
                    && fs::create_dir_all(parent).is_err()
                {
                    eprintln!("Error creating output directory for '{}'", out_path);
                    process::exit(1);
                }
                if let Err(e) = fs::write(out_path, &output) {
                    eprintln!("Error writing output file '{}': {}", out_path, e);
                    process::exit(1);
                }
                eprintln!(
                    "Processed {} -> {} records, output: {}",
                    input_count, output_count, out_path
                );
            } else {
                // Write to stdout
                if let Err(e) = io::stdout().write_all(output.as_bytes()) {
                    eprintln!("Error writing output: {}", e);
                    process::exit(1);
                }
                if !output.is_empty() && !output.ends_with('\n') {
                    println!();
                }
                eprintln!("Processed {} -> {} records", input_count, output_count);
            }
        }
        Err(e) => {
            eprintln!("Pipeline error: {}", e);
            process::exit(1);
        }
    }
}
