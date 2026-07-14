//! Local trace store (JSONL) + trace CI rules — capture v0

mod branches;
mod capture;
mod capture_scope;
mod check;
mod claude_code;
mod context_read;
mod cursor;
mod pr_appendix;
mod recording;
mod redact;
mod session_graph;
mod store;
mod transcripts;

pub use branches::{
    create_capture_branch, get_branch, list_branches_for_workspace, merge_capture_branch,
    slugify_branch_label, CaptureBranchRecord, BranchStatus,
};
pub use capture_scope::{
    begin_capture_session, capture_log_boundary_available, capture_scope_label,
    detect_capture_scope, list_picker_candidates, list_transcript_candidates,
    load_workspace_capture_prefs, remember_scope_for_workspace, resolve_capture_log_boundary,
    resolve_capture_scope, save_workspace_capture_prefs, CaptureScopeResolution,
    CaptureStartResult, LastCaptureBoundary, StartCaptureOutcome, TranscriptCandidate,
    WorkspaceCapturePrefs, DEFAULT_RECENT_SECS, AMBIGUOUS_MTIME_SECS, PICKER_CANDIDATE_LIMIT,
    PICKER_LOOKBACK_SECS,
};
pub use capture::{
    bindings_path, contextlayer_dir, default_capture_root, load_bindings, load_recorder_state,
    save_bindings, save_recorder_state, CaptureCommit, CaptureMeta, CaptureStore, CaptureSummary,
    ContextCommitWindow, ContextLogWindow, LogMessage, LogReadLimits, ProjectBindings,
    RecorderFileState, RecorderState,
};
pub use context_read::{build_context_summary, find_checkpoint, CheckpointDetail, ContextSummary};
pub use check::{run_trace_check, TraceCheckInput, TraceCheckReport, TraceRules};
pub use cursor::{
    composer_chat_title, cursor_projects_root, discover_transcript_files,
    format_transcript_scope_label, import_session_log, import_transcript_file,
    import_transcript_text, parse_transcript_line, poll_cursor_transcripts, sanitize_project_key,
    IngestStats, ParsedTranscriptLine,
};
pub use claude_code::{
    claude_chat_title, claude_projects_root, claude_session_id_from_path,
    discover_claude_transcript_files, format_claude_scope_label, parse_claude_transcript_line,
    poll_claude_transcripts,
};
pub use transcripts::{
    discover_all_transcript_files, format_scope_label, parse_transcript_line_any,
    poll_all_transcripts, project_key_from_transcript_path, TranscriptSource,
};
pub use pr_appendix::{
    compile_pr_trace_appendix, compile_pr_trace_appendix_with_limits,
    compile_pr_trace_appendix_with_options, parse_log_slice_mode, LogSliceMode,
    PrTraceAppendixOptions, DEFAULT_LOG_SLICE, SINCE_CAPTURE_LOG_MAX,
};
pub use recording::{
    active_capture_branch_slug, list_active_sessions, load_capture_sessions,
    matching_capture_session, prune_unscoped_capture_sessions, recorder_state_key,
    save_capture_sessions, session_allows_transcript, session_message_count, start_capture_session,
    stop_capture_session, stop_capture_session_by_id, CaptureSession, CaptureSessionStore,
};
pub use redact::redact_secrets;
pub use session_graph::{
    build_session_graph, SessionGraph, SessionGraphLane, SessionGraphRow,
};
pub use store::{
    default_traces_dir, CheckpointCommit, TraceEvent, TraceStore, TraceSummary,
};
