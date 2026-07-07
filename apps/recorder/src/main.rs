//! contextlayer-recorder — live capture from Cursor agent-transcripts (opt-in sessions only).

use std::path::PathBuf;
use std::process::ExitCode;
use std::thread;
use std::time::Duration;

use clap::{Parser, Subcommand};
use contextlayer_db::{default_db_path, GraphStore};
use contextlayer_trace::{
    create_capture_branch, import_transcript_file, list_active_sessions, list_branches_for_workspace,
    load_bindings, load_recorder_state, merge_capture_branch, poll_cursor_transcripts,
    save_bindings, save_recorder_state, start_capture_session, stop_capture_session, CaptureStore,
};

#[derive(Parser)]
#[command(
    name = "contextlayer-recorder",
    about = "ContextLayer live capture — tails Cursor transcripts when a capture session is active"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Poll Cursor transcripts continuously (ingests only while capture session is active)
    Watch {
        #[arg(long, default_value_t = 2)]
        interval_secs: u64,
    },
    /// One-shot poll (respects active capture sessions)
    Once,
    /// Start an opt-in capture session (baseline — no backfill of old chat)
    Start {
        /// Workspace UUID or exact name from the desktop app
        #[arg(long)]
        workspace: String,
        #[arg(long)]
        cursor_project: Option<String>,
        #[arg(long)]
        transcript: Option<PathBuf>,
        #[arg(long)]
        label: Option<String>,
    },
    /// Stop capture for a workspace
    Stop {
        /// Workspace UUID or exact name
        #[arg(long)]
        workspace: String,
    },
    /// List active capture sessions
    Status,
    /// Bind a Cursor sanitized project folder to a ContextLayer workspace
    BindCursorProject {
        #[arg(long)]
        cursor_project: String,
        /// Workspace UUID or exact name
        #[arg(long)]
        workspace: String,
    },
    /// Bind an absolute repo path (also registers sanitized cursor project key)
    BindRepo {
        #[arg(long)]
        path: PathBuf,
        /// Workspace UUID or exact name
        #[arg(long)]
        workspace: String,
    },
    /// Import a transcript JSONL file into workspace log (onboarding — always explicit)
    Import {
        /// Workspace UUID or exact name
        #[arg(long)]
        workspace: String,
        #[arg(long)]
        file: PathBuf,
    },
    /// List workspaces (id + name) for CLI reference
    ListWorkspaces,
    /// List current project bindings
    ListBindings,
    /// Fork capture to a branch subfolder (main log frozen; same watch session)
    Branch {
        #[arg(long)]
        workspace: String,
        #[arg(long)]
        label: String,
    },
    /// Merge a capture branch (confirmed | rejected)
    Merge {
        #[arg(long)]
        branch_id: String,
        #[arg(long)]
        outcome: String,
    },
    /// List capture branches for a workspace
    ListBranches {
        #[arg(long)]
        workspace: String,
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

fn resolve_workspace(name_or_id: &str) -> Result<String, String> {
    let store = GraphStore::open(&default_db_path()).map_err(|e| e.to_string())?;
    store
        .resolve_workspace_id(name_or_id)
        .map_err(|e| e.to_string())
}

fn run() -> Result<ExitCode, String> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Watch { interval_secs } => {
            eprintln!(
                "contextlayer-recorder: polling every {interval_secs}s — ingest only when `start` session is active (Ctrl+C to stop)"
            );
            loop {
                let stats = poll_once()?;
                if stats.messages_appended > 0 {
                    eprintln!(
                        "ingested {} message(s) from {} file(s)",
                        stats.messages_appended, stats.files_scanned
                    );
                }
                thread::sleep(Duration::from_secs(interval_secs));
            }
        }
        Commands::Once => {
            let stats = poll_once()?;
            println!(
                "scanned {} file(s), appended {} message(s), skipped {} unbound, skipped {} gated (no session)",
                stats.files_scanned,
                stats.messages_appended,
                stats.files_skipped_unbound,
                stats.files_skipped_gated
            );
            Ok(ExitCode::from(0))
        }
        Commands::Start {
            workspace,
            cursor_project,
            transcript,
            label,
        } => {
            let workspace_id = resolve_workspace(&workspace)?;
            let transcript_path = transcript.map(|p| p.to_string_lossy().to_string());
            let (session, baselined) = start_capture_session(
                &workspace_id,
                cursor_project,
                transcript_path,
                label,
            )?;
            println!(
                "capture session `{}` started for `{workspace}` → `{workspace_id}` (baselined {baselined} transcript file(s))",
                session.id
            );
            Ok(ExitCode::from(0))
        }
        Commands::Stop { workspace } => {
            let workspace_id = resolve_workspace(&workspace)?;
            match stop_capture_session(&workspace_id)? {
                Some(s) => {
                    println!("stopped capture session `{}` for `{workspace}`", s.id);
                }
                None => {
                    println!("no active capture session for `{workspace}`");
                }
            }
            Ok(ExitCode::from(0))
        }
        Commands::Status => {
            let sessions = list_active_sessions()?;
            if sessions.is_empty() {
                println!("no active capture sessions");
            } else {
                println!("{}", serde_json::to_string_pretty(&sessions).unwrap_or_default());
            }
            Ok(ExitCode::from(0))
        }
        Commands::BindCursorProject {
            cursor_project,
            workspace,
        } => {
            let workspace_id = resolve_workspace(&workspace)?;
            let mut bindings = load_bindings()?;
            bindings
                .cursor_projects
                .insert(cursor_project.clone(), workspace_id.clone());
            save_bindings(&bindings)?;
            println!("bound cursor project `{cursor_project}` → `{workspace}` (`{workspace_id}`)");
            Ok(ExitCode::from(0))
        }
        Commands::BindRepo { path, workspace } => {
            let workspace_id = resolve_workspace(&workspace)?;
            let abs = path
                .canonicalize()
                .map_err(|e| format!("resolve path: {e}"))?;
            let key = contextlayer_trace::sanitize_project_key(&abs.to_string_lossy());
            let mut bindings = load_bindings()?;
            bindings
                .repo_paths
                .insert(abs.to_string_lossy().to_string(), workspace_id.clone());
            bindings.cursor_projects.insert(key.clone(), workspace_id.clone());
            save_bindings(&bindings)?;
            println!(
                "bound repo `{}` (cursor key `{key}`) → `{workspace}` (`{workspace_id}`)",
                abs.display()
            );
            Ok(ExitCode::from(0))
        }
        Commands::Import { workspace, file } => {
            let workspace_id = resolve_workspace(&workspace)?;
            let capture = CaptureStore::default_open()?;
            let n = import_transcript_file(&capture, &workspace_id, &file)?;
            println!("imported {n} message(s) into `{workspace}` log");
            Ok(ExitCode::from(0))
        }
        Commands::ListWorkspaces => {
            let store = GraphStore::open(&default_db_path()).map_err(|e| e.to_string())?;
            for ws in store.list_workspaces(false).map_err(|e| e.to_string())? {
                println!("{}  {}", ws.id, ws.name);
            }
            Ok(ExitCode::from(0))
        }
        Commands::ListBindings => {
            let bindings = load_bindings()?;
            println!("{}", serde_json::to_string_pretty(&bindings).unwrap_or_default());
            Ok(ExitCode::from(0))
        }
        Commands::Branch { workspace, label } => {
            let workspace_id = resolve_workspace(&workspace)?;
            let capture = CaptureStore::default_open()?;
            let record = create_capture_branch(&capture, &workspace_id, &label)?;
            println!(
                "branch `{}` ({}) — capture line switched to `{}` (main frozen at seq {})",
                record.label, record.id, record.slug, record.main_log_seq_at_fork
            );
            Ok(ExitCode::from(0))
        }
        Commands::Merge { branch_id, outcome } => {
            let capture = CaptureStore::default_open()?;
            let merged = merge_capture_branch(&capture, &branch_id, &outcome)?;
            println!(
                "branch `{}` merged — status: {}",
                merged.label, merged.status
            );
            Ok(ExitCode::from(0))
        }
        Commands::ListBranches { workspace } => {
            let workspace_id = resolve_workspace(&workspace)?;
            let capture = CaptureStore::default_open()?;
            let branches = list_branches_for_workspace(&capture, &workspace_id)?;
            println!("{}", serde_json::to_string_pretty(&branches).unwrap_or_default());
            Ok(ExitCode::from(0))
        }
    }
}

fn poll_once() -> Result<contextlayer_trace::IngestStats, String> {
    let capture = CaptureStore::default_open()?;
    let bindings = load_bindings()?;
    let mut state = load_recorder_state()?;
    let stats = poll_cursor_transcripts(&capture, &bindings, &mut state)?;
    save_recorder_state(&state)?;
    Ok(stats)
}
