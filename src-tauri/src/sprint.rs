use serde::{Deserialize, Serialize};

/// A sprint is a burst of continuous typing.
/// It starts with the first keypress and ends when the user is idle
/// for longer than the configured timeout.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sprint {
    /// Timestamp of the first keypress in this sprint (ms since epoch)
    pub start_time: f64,
    /// Timestamp of the last keypress in this sprint (ms since epoch)
    pub end_time: f64,
    /// Number of characters typed correctly
    pub hits: u64,
    /// Number of characters missed
    pub misses: u64,
    /// Characters typed in this sprint
    pub characters: Vec<SprintChar>,
    /// Words per minute calculated from this sprint
    pub wpm: f64,
    /// Accuracy as a ratio [0.0, 1.0]
    pub accuracy: f64,
}

/// A single character typed during a sprint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SprintChar {
    pub expected: char,
    pub actual: char,
    pub correct: bool,
    pub timestamp: f64,
    pub latency_ms: f64,
}

/// Sprint detector that tracks keystrokes and detects idle periods.
#[derive(Debug)]
pub struct SprintDetector {
    /// Idle timeout in milliseconds (default: 3000ms = 3s)
    pub idle_timeout_ms: f64,
    /// Characters in the current sprint
    current_chars: Vec<SprintChar>,
    /// Timestamp of the last keypress
    last_keypress_time: f64,
    /// Whether we're currently in a sprint
    is_active: bool,
    /// Recent WPM values for rolling average
    recent_wpms: Vec<f64>,
}

impl SprintDetector {
    pub fn new(idle_timeout_ms: f64) -> Self {
        Self {
            idle_timeout_ms,
            current_chars: Vec::new(),
            last_keypress_time: 0.0,
            is_active: false,
            recent_wpms: Vec::new(),
        }
    }

    /// Record a keypress. Returns a completed Sprint if the idle timeout was exceeded.
    pub fn record_keypress(
        &mut self,
        timestamp: f64,
        expected: char,
        actual: char,
    ) -> Option<Sprint> {
        let latency = if self.is_active {
            timestamp - self.last_keypress_time
        } else {
            0.0
        };

        // Check if we should end the previous sprint due to idle timeout
        let completed_sprint = if self.is_active
            && timestamp - self.last_keypress_time > self.idle_timeout_ms
        {
            self.finalize_sprint()
        } else {
            None
        };

        // Start or continue the sprint
        if !self.is_active {
            self.is_active = true;
            self.current_chars.clear();
        }

        let correct = expected == actual;
        self.current_chars.push(SprintChar {
            expected,
            actual,
            correct,
            timestamp,
            latency_ms: latency,
        });

        self.last_keypress_time = timestamp;

        completed_sprint
    }

    /// Force-end the current sprint (e.g., when the user switches modes).
    pub fn force_end(&mut self) -> Option<Sprint> {
        if self.is_active && !self.current_chars.is_empty() {
            self.finalize_sprint()
        } else {
            None
        }
    }

    /// Finalize the current sprint and return it.
    fn finalize_sprint(&mut self) -> Option<Sprint> {
        if self.current_chars.is_empty() {
            self.is_active = false;
            return None;
        }

        let start_time = self.current_chars.first().unwrap().timestamp;
        let end_time = self.last_keypress_time;
        let duration_ms = end_time - start_time;

        let hits = self.current_chars.iter().filter(|c| c.correct).count() as u64;
        let misses = self.current_chars.iter().filter(|c| !c.correct).count() as u64;
        let total_chars = hits + misses;

        // WPM = (characters / 5) / (duration in minutes)
        // Standard: 1 word = 5 characters
        let wpm = if duration_ms > 0.0 {
            (hits as f64 / 5.0) / (duration_ms / 60000.0)
        } else {
            0.0
        };

        let accuracy = if total_chars > 0 {
            hits as f64 / total_chars as f64
        } else {
            0.0
        };

        // Track WPM for rolling average
        self.recent_wpms.push(wpm);
        if self.recent_wpms.len() > 10 {
            self.recent_wpms.remove(0);
        }

        let sprint = Sprint {
            start_time,
            end_time,
            hits,
            misses,
            characters: self.current_chars.clone(),
            wpm,
            accuracy,
        };

        self.current_chars.clear();
        self.is_active = false;

        Some(sprint)
    }

    /// Rolling average WPM from recent sprints.
    pub fn avg_wpm(&self) -> f64 {
        if self.recent_wpms.is_empty() {
            0.0
        } else {
            self.recent_wpms.iter().sum::<f64>() / self.recent_wpms.len() as f64
        }
    }

    /// Is a sprint currently active?
    pub fn is_sprint_active(&self) -> bool {
        self.is_active
    }
}
