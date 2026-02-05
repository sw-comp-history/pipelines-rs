//! CLI tool to run pipeline (.pipe) files using the record-at-a-time executor.

use clap::Parser;
use naive_pipe::execute_pipeline_rat;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::process;

/// Run a pipeline file against input data (record-at-a-time executor).
///
/// Produces identical output to pipe-run (batched) for all pipelines.
#[derive(Parser)]
#[command(name = "pipe-run-rat")]
struct Cli {
    /// Pipeline definition file (.pipe)
    pipeline: String,

    /// Input data file (80-byte fixed-width records, or /dev/stdin)
    input: String,

    /// Write output to file instead of stdout
    #[arg(short, long)]
    output: Option<String>,

    /// Show paths, executor, and record counts on stderr
    #[arg(short, long)]
    verbose: bool,
}

fn main() {
    let cli = Cli::parse();

    let pipeline_text = match fs::read_to_string(&cli.pipeline) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading pipeline file '{}': {e}", cli.pipeline);
            process::exit(1);
        }
    };

    let input_text = match fs::read_to_string(&cli.input) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading input file '{}': {e}", cli.input);
            process::exit(1);
        }
    };

    if cli.verbose {
        eprintln!("Pipeline: {}", cli.pipeline);
        eprintln!("Input:    {}", cli.input);
        eprintln!("Output:   {}", cli.output.as_deref().unwrap_or("(stdout)"));
        eprintln!("Executor: record-at-a-time");
    }

    match execute_pipeline_rat(&input_text, &pipeline_text) {
        Ok((output, input_count, output_count)) => {
            if let Some(out_path) = &cli.output {
                if let Some(parent) = Path::new(out_path.as_str()).parent()
                    && !parent.as_os_str().is_empty()
                    && fs::create_dir_all(parent).is_err()
                {
                    eprintln!("Error creating output directory for '{out_path}'");
                    process::exit(1);
                }
                if let Err(e) = fs::write(out_path, &output) {
                    eprintln!("Error writing output file '{out_path}': {e}");
                    process::exit(1);
                }
            } else {
                if let Err(e) = io::stdout().write_all(output.as_bytes()) {
                    eprintln!("Error writing output: {e}");
                    process::exit(1);
                }
                if !output.is_empty() && !output.ends_with('\n') {
                    println!();
                }
            }
            if cli.verbose {
                eprintln!("Records:  {input_count} in -> {output_count} out");
            }
        }
        Err(e) => {
            eprintln!("Pipeline error: {e}");
            process::exit(1);
        }
    }
}
