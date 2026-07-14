use std::io::{self, Write};
use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};
use cli::commands::{classes, disasm, dump, info, manifest, search, stats, strings};
use engine::Analyzer;

#[derive(Debug, Parser)]
#[command(
    name = "rutken",
    version,
    about = "Rust Android APK inspection toolkit"
)]
struct Cli {
    apk: PathBuf,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Info,
    Manifest,
    Strings {
        #[arg(long)]
        grep: Option<String>,
    },
    Classes,
    Search {
        query: String,
    },
    Disasm {
        query: String,
    },
    Dump {
        #[arg(long)]
        json: bool,

        #[arg(long)]
        raw: bool,

        #[arg(long, value_enum)]
        include: Vec<DumpInclude>,
    },
    /// Coarse analysis summary (classes/methods/instructions/XREF/CFG
    /// counts, timing, peak RAM). Hidden: a development/diagnostic command,
    /// not part of the stable product surface.
    #[command(hide = true)]
    Stats,
}

#[derive(Debug, Clone, ValueEnum)]
enum DumpInclude {
    Strings,
}

fn main() {
    let cli = Cli::parse();
    let start = std::time::Instant::now();

    let ir = match Analyzer::open(&cli.apk).and_then(|analyzer| analyzer.analyze()) {
        Ok(ir) => ir,
        Err(err) => {
            eprintln!("Error: {}", err);
            std::process::exit(1);
        }
    };

    match cli.command {
        Command::Info => {
            let report = info::collect(&ir);
            write_line(&format!("SHA256: {}", report.sha256));
            write_line(&format!("Size: {}MB", bytes_to_mb(report.size)));
            write_line(&format!("DEX files: {}", report.dex_files.len()));
            write_line(&format!("Classes: {}", report.classes));
            write_line(&format!("Native: {}", report.native));

            if let Some(package) = report.package {
                write_line(&format!("Package: {}", package));
            }
            if let Some(min_sdk) = report.min_sdk {
                write_line(&format!("Min SDK: {}", min_sdk));
            }
            if let Some(target_sdk) = report.target_sdk {
                write_line(&format!("Target SDK: {}", target_sdk));
            }

            if !report.dex_files.is_empty() {
                write_line("DEX:");
                for dex in report.dex_files {
                    write_line(&format!(" {}", dex));
                }
            }

            if !report.architectures.is_empty() {
                write_line("Architecture:");
                for architecture in report.architectures {
                    write_line(&format!(" {}", architecture));
                }
            }
        }
        Command::Manifest => {
            let output = manifest::render(&ir);
            write_all(&output);
        }
        Command::Strings { grep } => {
            let values = strings::collect(&ir, grep.as_deref());
            for value in values {
                write_line(&value);
            }
        }
        Command::Classes => {
            let output = classes::format(&ir);
            write_all(&output);
        }
        Command::Search { query } => {
            let matches = search::collect(&ir, &query);
            write_all(&search::format(&matches));
        }
        Command::Disasm { query } => {
            let output = disasm::render(&ir, &query);
            write_all(&output);
        }
        Command::Dump { json, raw, include } => {
            let include_strings = include
                .iter()
                .any(|value| matches!(value, DumpInclude::Strings));

            if raw {
                let report = dump::build_raw(&ir);
                if json {
                    match serde_json::to_string_pretty(&report) {
                        Ok(json) => write_line(&json),
                        Err(err) => {
                            eprintln!("Error: {}", err);
                            std::process::exit(1);
                        }
                    }
                } else {
                    write_line(&format!("{:#?}", report));
                }
            } else {
                let report = dump::build(&ir, include_strings);
                if json {
                    match serde_json::to_string_pretty(&report) {
                        Ok(json) => write_line(&json),
                        Err(err) => {
                            eprintln!("Error: {}", err);
                            std::process::exit(1);
                        }
                    }
                } else {
                    write_line(&format!("{:#?}", report));
                }
            }
        }
        Command::Stats => {
            let report = stats::collect(&ir);
            let elapsed = start.elapsed();
            let peak_rss_kb = stats::peak_rss_kb();
            write_all(&stats::render(&report, elapsed, peak_rss_kb));
        }
    }
}

fn bytes_to_mb(bytes: u64) -> u64 {
    bytes / (1024 * 1024)
}

fn write_all(text: &str) {
    let mut stdout = io::stdout().lock();
    if let Err(err) = stdout
        .write_all(text.as_bytes())
        .and_then(|_| stdout.flush())
    {
        handle_stdout_error(err);
    }
}

fn write_line(text: &str) {
    let mut stdout = io::stdout().lock();
    if let Err(err) = stdout
        .write_all(text.as_bytes())
        .and_then(|_| stdout.write_all(b"\n"))
        .and_then(|_| stdout.flush())
    {
        handle_stdout_error(err);
    }
}

fn handle_stdout_error(err: io::Error) {
    if err.kind() == io::ErrorKind::BrokenPipe {
        std::process::exit(0);
    }

    eprintln!("Error: {}", err);
    std::process::exit(1);
}
