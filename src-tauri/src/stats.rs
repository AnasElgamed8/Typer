use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Per-key statistics tracked during a typing session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyStats {
    /// Total times this key was pressed correctly
    pub hit_count: u64,
    /// Total times this key was missed (wrong key pressed)
    pub miss_count: u64,
    /// Sum of all inter-key latencies for this key (in ms)
    pub total_latency_ms: f64,
    /// Best (fastest) latency for this key (in ms)
    pub best_latency_ms: f64,
    /// Recent latencies for rolling average (last 20 presses)
    pub recent_latencies: Vec<f64>,
}

impl KeyStats {
    pub fn new() -> Self {
        Self {
            hit_count: 0,
            miss_count: 0,
            total_latency_ms: 0.0,
            best_latency_ms: f64::MAX,
            recent_latencies: Vec::new(),
        }
    }

    /// Record a successful keypress with the given latency.
    pub fn record_hit(&mut self, latency_ms: f64) {
        self.hit_count += 1;
        self.total_latency_ms += latency_ms;
        if latency_ms < self.best_latency_ms {
            self.best_latency_ms = latency_ms;
        }
        self.recent_latencies.push(latency_ms);
        if self.recent_latencies.len() > 20 {
            self.recent_latencies.remove(0);
        }
    }

    /// Record a missed keypress.
    pub fn record_miss(&mut self) {
        self.miss_count += 1;
    }

    /// Average latency across all hits.
    pub fn avg_latency(&self) -> f64 {
        if self.hit_count == 0 {
            0.0
        } else {
            self.total_latency_ms / self.hit_count as f64
        }
    }

    /// Recent average latency (last 20 presses).
    pub fn recent_avg_latency(&self) -> f64 {
        if self.recent_latencies.is_empty() {
            0.0
        } else {
            self.recent_latencies.iter().sum::<f64>() / self.recent_latencies.len() as f64
        }
    }

    /// Accuracy as a ratio [0.0, 1.0].
    #[allow(dead_code)]
    pub fn accuracy(&self) -> f64 {
        let total = self.hit_count + self.miss_count;
        if total == 0 {
            0.0
        } else {
            self.hit_count as f64 / total as f64
        }
    }

    /// Total presses (hits + misses).
    pub fn total_presses(&self) -> u64 {
        self.hit_count + self.miss_count
    }
}

/// Global typing statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypingStats {
    /// Per-character stats
    pub key_stats: HashMap<char, KeyStats>,
    /// Total characters typed correctly
    pub total_hits: u64,
    /// Total characters missed
    pub total_misses: u64,
    /// Total sprints completed
    pub sprint_count: u64,
    /// Words per minute (rolling average of recent sprints)
    pub current_wpm: f64,
}

impl TypingStats {
    pub fn new() -> Self {
        Self {
            key_stats: HashMap::new(),
            total_hits: 0,
            total_misses: 0,
            sprint_count: 0,
            current_wpm: 0.0,
        }
    }

    /// Get or create stats for a character.
    pub fn get_or_create(&mut self, ch: char) -> &mut KeyStats {
        self.key_stats.entry(ch).or_insert_with(KeyStats::new)
    }

    /// Record a successful keypress.
    pub fn record_hit(&mut self, ch: char, latency_ms: f64) {
        self.total_hits += 1;
        self.get_or_create(ch).record_hit(latency_ms);
    }

    /// Record a missed keypress.
    pub fn record_miss(&mut self, expected: char) {
        self.total_misses += 1;
        self.get_or_create(expected).record_miss();
    }

    /// Overall accuracy.
    pub fn accuracy(&self) -> f64 {
        let total = self.total_hits + self.total_misses;
        if total == 0 {
            0.0
        } else {
            self.total_hits as f64 / total as f64
        }
    }

    /// Get heatmap data: character -> press count.
    pub fn heatmap_data(&self) -> HashMap<char, u64> {
        self.key_stats
            .iter()
            .map(|(&ch, stats)| (ch, stats.total_presses()))
            .collect()
    }
}
