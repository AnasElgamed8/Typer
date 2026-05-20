mod keyboard;
mod stats;
mod sprint;
mod adaptive;

use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::State;

/// Application state shared between Rust and frontend via Tauri IPC.
pub struct AppState {
    pub stats: Mutex<stats::TypingStats>,
    pub sprint_detector: Mutex<sprint::SprintDetector>,
    pub lesson: Mutex<adaptive::AdaptiveLesson>,
    pub current_text: Mutex<String>,
    pub settings: Mutex<AppSettings>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub idle_timeout_ms: f64,
    pub target_wpm: f64,
    pub word_count: usize,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            idle_timeout_ms: 3000.0,
            target_wpm: 40.0,
            word_count: 15,
        }
    }
}

/// Response returned after each keypress.
#[derive(Debug, Serialize)]
pub struct KeypressResult {
    /// Whether the keypress was correct
    pub correct: bool,
    /// Current position in the practice text
    pub position: usize,
    /// Current WPM (rolling average)
    pub wpm: f64,
    /// Current accuracy
    pub accuracy: f64,
    /// Completed sprint (if idle timeout was exceeded)
    pub completed_sprint: Option<SprintInfo>,
    /// Updated heatmap data
    pub heatmap: std::collections::HashMap<char, u64>,
    /// Per-key confidence scores
    pub confidence: std::collections::HashMap<char, f64>,
}

#[derive(Debug, Serialize)]
pub struct SprintInfo {
    pub wpm: f64,
    pub accuracy: f64,
    pub hits: u64,
    pub misses: u64,
    pub duration_ms: f64,
}

impl From<sprint::Sprint> for SprintInfo {
    fn from(s: sprint::Sprint) -> Self {
        Self {
            wpm: s.wpm,
            accuracy: s.accuracy,
            hits: s.hits,
            misses: s.misses,
            duration_ms: s.end_time - s.start_time,
        }
    }
}

/// Record a keypress and return the result.
#[tauri::command]
fn record_keypress(
    key: String,
    shift: bool,
    timestamp: f64,
    state: State<AppState>,
) -> Result<KeypressResult, String> {
    let ch = keyboard::js_key_to_char(&key, shift)
        .ok_or_else(|| format!("Unhandled key: {}", key))?;

    let text = state.current_text.lock().unwrap().clone();
    let mut stats = state.stats.lock().unwrap();
    let mut detector = state.sprint_detector.lock().unwrap();
    let mut lesson = state.lesson.lock().unwrap();

    // Get the expected character at the current position
    let position = stats.total_hits + stats.total_misses;
    let expected = text.chars().nth(position as usize).unwrap_or(' ');

    let correct = ch == expected;

    // Record in sprint detector
    let completed_sprint = detector.record_keypress(timestamp, expected, ch);

    // If a sprint completed, update stats
    if let Some(ref sprint) = completed_sprint {
        stats.sprint_count += 1;
        stats.current_wpm = detector.avg_wpm();

        // Record individual key hits/misses from the sprint
        for sc in &sprint.characters {
            if sc.correct {
                stats.record_hit(sc.expected, sc.latency_ms);
            } else {
                stats.record_miss(sc.expected);
            }
        }

        // Update adaptive lesson alphabet
        lesson.update_alphabet(&stats);
    }

    // Record the current keypress in stats too (for real-time feedback)
    // Note: sprint characters are already counted when sprint completes,
    // so we only update the rolling average here, not the hit/miss counts.

    // Build heatmap
    let heatmap = stats.heatmap_data();

    // Build confidence map
    let mut confidence = std::collections::HashMap::new();
    for &ch in lesson.active_keys() {
        confidence.insert(ch, lesson.key_confidence(ch, &stats));
    }

    let new_position = position + 1;

    Ok(KeypressResult {
        correct,
        position: new_position,
        wpm: detector.avg_wpm(),
        accuracy: stats.accuracy(),
        completed_sprint: completed_sprint.map(SprintInfo::from),
        heatmap,
        confidence,
    })
}

/// Generate a new lesson text based on the adaptive algorithm.
#[tauri::command]
fn get_lesson(state: State<AppState>) -> Result<String, String> {
    let stats = state.stats.lock().unwrap();
    let mut lesson = state.lesson.lock().unwrap();
    let settings = state.settings.lock().unwrap();

    lesson.update_alphabet(&stats);
    let text = lesson.generate_lesson_text(&stats, settings.word_count);

    drop(stats);
    drop(lesson);

    *state.current_text.lock().unwrap() = text.clone();

    Ok(text)
}

/// Get current typing statistics.
#[tauri::command]
fn get_stats(state: State<AppState>) -> Result<StatsResponse, String> {
    let stats = state.stats.lock().unwrap();
    let lesson = state.lesson.lock().unwrap();
    let detector = state.sprint_detector.lock().unwrap();

    let mut key_details: Vec<KeyDetail> = stats
        .key_stats
        .iter()
        .map(|(&ch, ks)| KeyDetail {
            character: ch,
            hit_count: ks.hit_count,
            miss_count: ks.miss_count,
            avg_latency_ms: ks.avg_latency(),
            recent_avg_latency_ms: ks.recent_avg_latency(),
            best_latency_ms: if ks.best_latency_ms == f64::MAX {
                0.0
            } else {
                ks.best_latency_ms
            },
            confidence: lesson.key_confidence(ch, &stats),
        })
        .collect();

    key_details.sort_by(|a, b| b.hit_count.cmp(&a.hit_count));

    Ok(StatsResponse {
        total_hits: stats.total_hits,
        total_misses: stats.total_misses,
        accuracy: stats.accuracy(),
        avg_wpm: detector.avg_wpm(),
        sprint_count: stats.sprint_count,
        active_keys: lesson.active_keys().to_vec(),
        mastered_keys: lesson.mastered_keys().to_vec(),
        key_details,
    })
}

#[derive(Serialize)]
pub struct StatsResponse {
    pub total_hits: u64,
    pub total_misses: u64,
    pub accuracy: f64,
    pub avg_wpm: f64,
    pub sprint_count: u64,
    pub active_keys: Vec<char>,
    pub mastered_keys: Vec<char>,
    pub key_details: Vec<KeyDetail>,
}

#[derive(Serialize)]
pub struct KeyDetail {
    pub character: char,
    pub hit_count: u64,
    pub miss_count: u64,
    pub avg_latency_ms: f64,
    pub recent_avg_latency_ms: f64,
    pub best_latency_ms: f64,
    pub confidence: f64,
}

/// Update settings.
#[tauri::command]
fn update_settings(
    idle_timeout_ms: Option<f64>,
    target_wpm: Option<f64>,
    word_count: Option<usize>,
    state: State<AppState>,
) -> Result<(), String> {
    let mut settings = state.settings.lock().unwrap();
    if let Some(t) = idle_timeout_ms {
        settings.idle_timeout_ms = t;
    }
    if let Some(w) = target_wpm {
        settings.target_wpm = w;
    }
    if let Some(c) = word_count {
        settings.word_count = c;
    }

    // Update sprint detector timeout
    let mut detector = state.sprint_detector.lock().unwrap();
    detector.idle_timeout_ms = settings.idle_timeout_ms;

    Ok(())
}

/// Reset all statistics and start fresh.
#[tauri::command]
fn reset_stats(state: State<AppState>) -> Result<(), String> {
    let settings = state.settings.lock().unwrap();
    let idle_timeout = settings.idle_timeout_ms;
    drop(settings);

    *state.stats.lock().unwrap() = stats::TypingStats::new();
    *state.sprint_detector.lock().unwrap() = sprint::SprintDetector::new(idle_timeout);
    *state.lesson.lock().unwrap() = adaptive::AdaptiveLesson::new();
    *state.current_text.lock().unwrap() = String::new();

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState {
            stats: Mutex::new(stats::TypingStats::new()),
            sprint_detector: Mutex::new(sprint::SprintDetector::new(3000.0)),
            lesson: Mutex::new(adaptive::AdaptiveLesson::new()),
            current_text: Mutex::new(String::new()),
            settings: Mutex::new(AppSettings::default()),
        })
        .invoke_handler(tauri::generate_handler![
            record_keypress,
            get_lesson,
            get_stats,
            update_settings,
            reset_stats,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
