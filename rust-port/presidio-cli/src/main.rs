use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use indicatif::{ProgressBar, ProgressStyle};
use presidio_analyzer::AnalyzerEngine;
use presidio_common::{EntityType, Language};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};
use walkdir::WalkDir;

#[derive(Parser, Debug)]
#[command(name = "presidio")]
#[command(author = "Presidio Contributors")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Scan files and directories for PII (Personally Identifiable Information)", long_about = None)]
struct Cli {
    /// Path to scan (file or directory)
    #[arg(value_name = "PATH")]
    path: PathBuf,

    /// Language of the text (default: en)
    #[arg(short, long, default_value = "en")]
    language: String,

    /// Specific entity types to detect (comma-separated)
    #[arg(short, long, value_delimiter = ',')]
    entities: Option<Vec<String>>,

    /// Minimum score threshold (0.0 to 1.0)
    #[arg(short, long, default_value = "0.5")]
    threshold: f32,

    /// Output format
    #[arg(short, long, value_enum, default_value = "standard")]
    output: OutputFormat,

    /// File extensions to scan (default: txt,md,json,csv)
    #[arg(long, value_delimiter = ',', default_value = "txt,md,json,csv,log")]
    extensions: Vec<String>,

    /// Maximum file size to scan in bytes (default: 10MB)
    #[arg(long, default_value = "10485760")]
    max_size: u64,

    /// Follow symbolic links
    #[arg(long)]
    follow_links: bool,

    /// Write results to JSON file
    #[arg(short, long)]
    json_output: Option<PathBuf>,

    /// Number of parallel threads (default: CPU count)
    #[arg(short, long)]
    threads: Option<usize>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum OutputFormat {
    /// Standard colored output
    Standard,
    /// GitHub Actions format
    Github,
    /// Parsable format (one finding per line)
    Parsable,
    /// JSON format
    Json,
}

#[derive(Debug, Serialize, Deserialize)]
struct ScanResult {
    file: PathBuf,
    findings: Vec<Finding>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Finding {
    entity_type: String,
    start: usize,
    end: usize,
    score: f32,
    line: usize,
    column: usize,
    text: String,
    context: String,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Set thread pool size if specified
    if let Some(threads) = cli.threads {
        rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build_global()
            .context("Failed to initialize thread pool")?;
    }

    // Parse language
    let language = Language::from_str(&cli.language)
        .ok_or_else(|| anyhow::anyhow!("Invalid language: {}", cli.language))?;

    // Parse entity types if specified
    let entities = if let Some(entity_strs) = &cli.entities {
        let mut parsed = Vec::new();
        for entity_str in entity_strs {
            let entity = EntityType::from_str(entity_str);
            parsed.push(entity);
        }
        Some(parsed)
    } else {
        None
    };

    // Create analyzer
    let analyzer = AnalyzerEngine::with_defaults();

    // Collect files to scan
    let files = collect_files(&cli.path, &cli.extensions, cli.max_size, cli.follow_links)?;

    if files.is_empty() {
        eprintln!("No files found to scan.");
        return Ok(());
    }

    // Create progress bar
    let pb = ProgressBar::new(files.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {msg}")?
            .progress_chars("=>-"),
    );

    // Scan files in parallel
    let results: Vec<ScanResult> = files
        .par_iter()
        .filter_map(|file_path| {
            let result = scan_file(&analyzer, file_path, language, entities.as_deref(), cli.threshold);
            pb.inc(1);
            match result {
                Ok(scan_result) => {
                    if !scan_result.findings.is_empty() {
                        Some(scan_result)
                    } else {
                        None
                    }
                }
                Err(e) => {
                    eprintln!("Error scanning {}: {}", file_path.display(), e);
                    None
                }
            }
        })
        .collect();

    pb.finish_with_message("Scan complete");

    // Output results
    match cli.output {
        OutputFormat::Standard => print_standard(&results)?,
        OutputFormat::Github => print_github(&results),
        OutputFormat::Parsable => print_parsable(&results),
        OutputFormat::Json => print_json(&results)?,
    }

    // Write JSON output if requested
    if let Some(json_path) = cli.json_output {
        let json = serde_json::to_string_pretty(&results)?;
        fs::write(&json_path, json)
            .with_context(|| format!("Failed to write JSON to {}", json_path.display()))?;
        eprintln!("Results written to {}", json_path.display());
    }

    // Exit with error code if PII was found
    if results.iter().any(|r| !r.findings.is_empty()) {
        std::process::exit(1);
    }

    Ok(())
}

fn collect_files(
    path: &Path,
    extensions: &[String],
    max_size: u64,
    follow_links: bool,
) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    if path.is_file() {
        files.push(path.to_path_buf());
    } else if path.is_dir() {
        let walker = WalkDir::new(path)
            .follow_links(follow_links)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter(|e| {
                if let Some(ext) = e.path().extension() {
                    extensions.contains(&ext.to_string_lossy().to_string())
                } else {
                    false
                }
            })
            .filter(|e| {
                e.metadata()
                    .map(|m| m.len() <= max_size)
                    .unwrap_or(false)
            });

        for entry in walker {
            files.push(entry.path().to_path_buf());
        }
    } else {
        anyhow::bail!("Path does not exist or is not accessible: {}", path.display());
    }

    Ok(files)
}

fn scan_file(
    analyzer: &AnalyzerEngine,
    file_path: &Path,
    language: Language,
    entities: Option<&[EntityType]>,
    threshold: f32,
) -> Result<ScanResult> {
    let content = fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read file: {}", file_path.display()))?;

    let results = analyzer
        .analyze(&content, language, entities, None, threshold, false)
        .with_context(|| format!("Failed to analyze file: {}", file_path.display()))?;

    let mut findings = Vec::new();
    for result in results {
        let (line, column) = get_line_column(&content, result.start);
        let context = get_context(&content, result.start, result.end, 40);
        let text = content[result.start..result.end].to_string();

        findings.push(Finding {
            entity_type: result.entity_type.to_string(),
            start: result.start,
            end: result.end,
            score: result.score,
            line,
            column,
            text,
            context,
        });
    }

    Ok(ScanResult {
        file: file_path.to_path_buf(),
        findings,
    })
}

fn get_line_column(text: &str, position: usize) -> (usize, usize) {
    let mut line = 1;
    let mut column = 1;

    for (i, ch) in text.char_indices() {
        if i >= position {
            break;
        }
        if ch == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }

    (line, column)
}

fn get_context(text: &str, start: usize, end: usize, context_len: usize) -> String {
    let context_start = start.saturating_sub(context_len);
    let context_end = (end + context_len).min(text.len());

    let mut context = text[context_start..context_end].to_string();
    context = context.replace('\n', " ").replace('\r', "");

    // Add indicators
    let pii_start = start - context_start;
    let pii_end = end - context_start;

    format!(
        "{}>>>{}<<<{}",
        &context[..pii_start],
        &context[pii_start..pii_end],
        &context[pii_end..]
    )
}

fn print_standard(results: &[ScanResult]) -> Result<()> {
    let mut stdout = StandardStream::stdout(ColorChoice::Auto);
    let mut total_findings = 0;

    for result in results {
        writeln!(
            stdout,
            "\n{}:",
            result.file.display()
        )?;

        for finding in &result.findings {
            total_findings += 1;

            // Print location
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::Yellow)))?;
            write!(stdout, "  {}:{}:{}",
                result.file.display(),
                finding.line,
                finding.column
            )?;
            stdout.reset()?;

            // Print entity type
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(true))?;
            write!(stdout, " {} ", finding.entity_type)?;
            stdout.reset()?;

            // Print score
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::Cyan)))?;
            writeln!(stdout, "(score: {:.2})", finding.score)?;
            stdout.reset()?;

            // Print context
            writeln!(stdout, "    {}", finding.context)?;
        }
    }

    writeln!(stdout)?;
    if total_findings > 0 {
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(true))?;
        writeln!(stdout, "Found {} PII entities in {} files", total_findings, results.len())?;
        stdout.reset()?;
    } else {
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green)))?;
        writeln!(stdout, "No PII found")?;
        stdout.reset()?;
    }

    Ok(())
}

fn print_github(results: &[ScanResult]) {
    for result in results {
        for finding in &result.findings {
            println!(
                "::error file={},line={},col={}::{} detected (score: {:.2}): {}",
                result.file.display(),
                finding.line,
                finding.column,
                finding.entity_type,
                finding.score,
                finding.text
            );
        }
    }
}

fn print_parsable(results: &[ScanResult]) {
    for result in results {
        for finding in &result.findings {
            println!(
                "{}:{}:{}:{}:{:.2}:{}",
                result.file.display(),
                finding.line,
                finding.column,
                finding.entity_type,
                finding.score,
                finding.text
            );
        }
    }
}

fn print_json(results: &[ScanResult]) -> Result<()> {
    let json = serde_json::to_string_pretty(&results)?;
    println!("{}", json);
    Ok(())
}
