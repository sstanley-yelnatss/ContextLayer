//! Local trace store (JSONL) + trace CI rules — capture v0

mod branches;
mod capture;
mod check;
mod context_read;
mod cursor;
mod pr_appendix;
mod recording;
mod redact;
mod store;

pub use branches::{
    create_capture_branch, get_branch, list_branches_for_workspace, merge_capture_branch,
    slugify_branch_label, CaptureBranchRecord, BranchStatus,
};
pub use capture::{
    bindings_path, default_capture_root, load_bindings, load_recorder_state, save_bindings,
    save_recorder_state, CaptureCommit, CaptureMeta, CaptureStore, CaptureSummary,
    ContextCommitWindow, ContextLogWindow, LogMessage, ProjectBindings, RecorderFileState,
    RecorderState,
};
pub use context_read::{build_context_summary, find_checkpoint, CheckpointDetail, ContextSummary};
pub use check::{run_trace_check, TraceCheckInput, TraceCheckReport, TraceRules};
pub use cursor::{
    cursor_projects_root, discover_transcript_files, import_session_log, import_transcript_file,
    import_transcript_text, parse_transcript_line, poll_cursor_transcripts, sanitize_project_key,
    IngestStats,
};
pub use pr_appendix::{
    compile_pr_trace_appendix, compile_pr_trace_appendix_with_limits,
    compile_pr_trace_appendix_with_options, PrTraceAppendixOptions,
};
pub use recording::{
    active_capture_branch_slug, list_active_sessions, load_capture_sessions, recorder_state_key,
    save_capture_sessions, start_capture_session, stop_capture_session,
    stop_capture_session_by_id, CaptureSession, CaptureSessionStore,
};
pub use redact::redact_secrets;
pub use store::{
    default_traces_dir, CheckpointCommit, TraceEvent, TraceStore, TraceSummary,
};
