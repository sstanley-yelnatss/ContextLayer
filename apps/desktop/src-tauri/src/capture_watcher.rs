//! Background Cursor transcript polling — same loop as `contextlayer-recorder watch`.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use contextlayer_trace::{
    load_bindings, load_recorder_state, poll_cursor_transcripts, save_recorder_state,
    CaptureStore,
};

struct WatcherState {
    stop: Option<Arc<AtomicBool>>,
    handle: Option<JoinHandle<()>>,
}

impl WatcherState {
    const fn new() -> Self {
        Self {
            stop: None,
            handle: None,
        }
    }
}

static WATCHER: Mutex<WatcherState> = Mutex::new(WatcherState::new());

fn poll_once() -> Result<contextlayer_trace::IngestStats, String> {
    let capture = CaptureStore::default_open()?;
    let bindings = load_bindings()?;
    let mut state = load_recorder_state()?;
    let stats = poll_cursor_transcripts(&capture, &bindings, &mut state)?;
    save_recorder_state(&state)?;
    Ok(stats)
}

pub fn ensure_running() {
    let mut guard = WATCHER.lock().expect("capture watcher lock");
    if guard.handle.is_some() {
        return;
    }

    let stop = Arc::new(AtomicBool::new(false));
    let thread_stop = Arc::clone(&stop);
    let handle = thread::spawn(move || {
        while !thread_stop.load(Ordering::Relaxed) {
            let _ = poll_once();
            thread::sleep(Duration::from_secs(2));
        }
    });

    guard.stop = Some(stop);
    guard.handle = Some(handle);
}

pub fn stop() {
    let mut guard = WATCHER.lock().expect("capture watcher lock");
    if let Some(stop) = guard.stop.take() {
        stop.store(true, Ordering::Relaxed);
    }
    if let Some(handle) = guard.handle.take() {
        let _ = handle.join();
    }
}

pub fn is_running() -> bool {
    WATCHER
        .lock()
        .map(|g| g.handle.is_some())
        .unwrap_or(false)
}
