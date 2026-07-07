//! contextlayer-trace — trace CI for PRs (capture v0 governance)

use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use contextlayer_trace::{run_trace_check, TraceCheckInput, TraceRules, TraceStore};

#[derive(Parser)]
#[command(name = "contextlayer-trace", about = "ContextLayer trace CI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Validate PR body and repo against trace rules
    Check {
        #[arg(long, default_value = ".contextlayer/rules.yml")]
        rules: PathBuf,
        #[arg(long, help = "Path to file containing PR description markdown")]
        pr_body: Option<PathBuf>,
        #[arg(long, help = "PR body text (overrides --pr-body file)")]
        pr_body_text: Option<String>,
        #[arg(long, default_value = ".")]
        repo: PathBuf,
        #[arg(long, help = "Workspace UUID for checkpoint rule")]
        workspace_id: Option<String>,
    },
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::from(1)
        }
    }
}

fn run() -> Result<ExitCode, String> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Check {
            rules,
            pr_body,
            pr_body_text,
            repo,
            workspace_id,
        } => {
            let rules = TraceRules::load(&rules)?;
            let body = if let Some(t) = pr_body_text {
                t
            } else if let Some(path) = pr_body {
                fs::read_to_string(&path).map_err(|e| format!("read pr body: {e}"))?
            } else {
                String::new()
            };

            let trace_store = TraceStore::default_open().ok();
            let input = TraceCheckInput {
                pr_body: body,
                repo_root: repo,
                scan_globs: vec![],
                workspace_id,
                trace_store,
            };

            let report = run_trace_check(&rules, &input);
            for w in &report.warnings {
                eprintln!("warning: {w}");
            }
            if report.passed {
                println!("trace CI: passed");
                Ok(ExitCode::from(0))
            } else {
                for e in &report.errors {
                    eprintln!("error: {e}");
                }
                eprintln!("trace CI: failed ({} error(s))", report.errors.len());
                Ok(ExitCode::from(1))
            }
        }
    }
}
