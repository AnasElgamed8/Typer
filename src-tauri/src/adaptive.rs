use crate::stats::TypingStats;
use rand::prelude::*;

/// The target typing speed in WPM. Keys that are typed at this speed
/// or faster are considered "learned" (confidence >= 1.0).
const DEFAULT_TARGET_WPM: f64 = 40.0;

/// Minimum number of keys in the active alphabet.
const MIN_ALPHABET_SIZE: usize = 6;

/// Word lists for lesson generation.
/// These are common English words ordered by frequency.
const COMMON_WORDS: &[&str] = &[
    "the", "be", "to", "of", "and", "a", "in", "that", "have", "I",
    "it", "for", "not", "on", "with", "he", "as", "you", "do", "at",
    "this", "but", "his", "by", "from", "they", "we", "say", "her", "she",
    "or", "an", "will", "my", "one", "all", "would", "there", "their", "what",
    "so", "up", "out", "if", "about", "who", "get", "which", "go", "me",
    "when", "make", "can", "like", "time", "no", "just", "him", "know", "take",
    "people", "into", "year", "your", "good", "some", "could", "them", "see",
    "other", "than", "then", "now", "look", "only", "come", "its", "over",
    "think", "also", "back", "after", "use", "two", "how", "our", "work",
    "first", "well", "way", "even", "new", "want", "because", "any", "these",
    "give", "day", "most", "us", "great", "between", "need", "large", "often",
    "hand", "high", "place", "hold", "free", "real", "life", "few", "north",
    "open", "seem", "together", "next", "white", "children", "begin", "got",
    "walk", "example", "ease", "paper", "group", "always", "music", "those",
    "both", "mark", "book", "letter", "until", "mile", "river", "car", "feet",
    "care", "second", "enough", "plain", "girl", "usual", "young", "ready",
    "above", "ever", "red", "list", "though", "feel", "talk", "bird", "soon",
    "body", "dog", "family", "direct", "pose", "leave", "song", "measure",
    "door", "product", "black", "short", "numeral", "class", "wind", "question",
    "happen", "complete", "ship", "area", "half", "rock", "order", "fire",
    "south", "problem", "piece", "told", "knew", "pass", "since", "top",
    "whole", "king", "space", "heard", "best", "hour", "better", "true",
    "during", "hundred", "five", "remember", "step", "early", "hold", "west",
    "ground", "interest", "reach", "fast", "verb", "sing", "listen", "six",
    "table", "travel", "less", "morning", "ten", "simple", "several", "vowel",
    "toward", "war", "lay", "against", "pattern", "slow", "center", "love",
    "person", "money", "serve", "appear", "road", "map", "rain", "rule",
    "govern", "pull", "cold", "notice", "voice", "energy", "hunt", "probable",
    "bed", "brother", "egg", "ride", "cell", "believe", "perhaps", "pick",
    "sudden", "count", "reason", "square", "moment", "develop", "catch",
    "sleep", "wonder", "machine", "program", "independent", "possible",
];

/// Adaptive lesson generator inspired by keybr.com's guided approach.
/// Introduces keys gradually and focuses on weak keys.
#[derive(Debug)]
pub struct AdaptiveLesson {
    /// Characters in the active alphabet (keys the user has been introduced to)
    active_keys: Vec<char>,
    /// Keys that have reached the target confidence
    mastered_keys: Vec<char>,
    /// The order in which keys should be introduced (QWERTY left-to-right, top-to-bottom)
    key_order: Vec<char>,
    /// Target WPM for considering a key "mastered"
    target_wpm: f64,
}

impl AdaptiveLesson {
    pub fn new() -> Self {
        // Key introduction order: home row first, then top row, then bottom row
        // This mirrors keybr.com's approach of teaching the most common keys first
        let key_order: Vec<char> = vec![
            // Home row (most important)
            'f', 'j', 'd', 'k', 's', 'l', 'a', ';', 'g', 'h',
            // Top row
            'r', 'u', 'e', 'i', 'w', 'o', 'q', 'p', 't', 'y',
            // Bottom row
            'v', 'm', 'c', ',', 'x', '.', 'z', '/', 'b', 'n',
            // Number row
            '4', '7', '3', '8', '2', '9', '1', '0', '5', '6',
            // Other
            '\'', '[', ']', '\\', '`', '-', '=', ' ',
        ];

        Self {
            active_keys: Vec::new(),
            mastered_keys: Vec::new(),
            key_order,
            target_wpm: DEFAULT_TARGET_WPM,
        }
    }

    /// Update the active alphabet based on current stats.
    /// This is called after each sprint to potentially introduce new keys.
    pub fn update_alphabet(&mut self, stats: &TypingStats) {
        // Ensure minimum alphabet size
        while self.active_keys.len() < MIN_ALPHABET_SIZE {
            if let Some(&next_key) = self.key_order.iter().find(|k| !self.active_keys.contains(k))
            {
                self.active_keys.push(next_key);
            } else {
                break;
            }
        }

        // Check if all active keys are confident enough to introduce a new one
        let all_confident = self.active_keys.iter().all(|&ch| {
            if let Some(key_stats) = stats.key_stats.get(&ch) {
                if key_stats.hit_count < 5 {
                    return false; // Not enough data
                }
                let avg_latency = key_stats.recent_avg_latency();
                if avg_latency <= 0.0 {
                    return false;
                }
                // Confidence = target_latency / actual_latency
                // Higher confidence = faster typing
                let target_latency = 60000.0 / (self.target_wpm * 5.0); // ms per char at target WPM
                let confidence = target_latency / avg_latency;
                confidence >= 1.0
            } else {
                false
            }
        });

        if all_confident {
            // Introduce the next key
            if let Some(&next_key) = self.key_order.iter().find(|k| !self.active_keys.contains(k))
            {
                self.active_keys.push(next_key);
            }
        }

        // Also check if any active keys should be marked as mastered
        for &ch in &self.active_keys {
            if self.mastered_keys.contains(&ch) {
                continue;
            }
            if let Some(key_stats) = stats.key_stats.get(&ch) {
                if key_stats.hit_count >= 20 {
                    let avg_latency = key_stats.recent_avg_latency();
                    if avg_latency > 0.0 {
                        let target_latency = 60000.0 / (self.target_wpm * 5.0);
                        let confidence = target_latency / avg_latency;
                        if confidence >= 1.0 {
                            self.mastered_keys.push(ch);
                        }
                    }
                }
            }
        }
    }

    /// Generate a practice text using only the active alphabet.
    /// Words containing characters not in the active alphabet are filtered out.
    /// Words containing weak keys are repeated more often.
    pub fn generate_lesson_text(&self, stats: &TypingStats, word_count: usize) -> String {
        let mut rng = thread_rng();

        // Filter words to only those using active keys
        let valid_words: Vec<&str> = COMMON_WORDS
            .iter()
            .filter(|word| {
                word.chars()
                    .all(|ch| self.active_keys.contains(&ch) || ch == ' ')
            })
            .copied()
            .collect();

        if valid_words.is_empty() {
            // Fallback: just repeat the active keys
            return self
                .active_keys
                .iter()
                .cycle()
                .take(word_count)
                .collect::<String>()
                .chars()
                .collect::<Vec<char>>()
                .chunks(5)
                .map(|chunk| chunk.iter().collect::<String>())
                .collect::<Vec<String>>()
                .join(" ");
        }

        // Build weighted word list: words with weak keys get higher weight
        let weighted_words: Vec<(&str, f64)> = valid_words
            .iter()
            .map(|&word| {
                let weight = word
                    .chars()
                    .map(|ch| {
                        if let Some(key_stats) = stats.key_stats.get(&ch) {
                            let avg = key_stats.recent_avg_latency();
                            if avg > 0.0 {
                                // Higher latency = higher weight = more practice
                                avg / 100.0
                            } else {
                                1.0 // Unknown key, normal weight
                            }
                        } else {
                            1.5 // Never typed, slightly higher weight
                        }
                    })
                    .fold(1.0f64, |a, b| a + b);
                (word, weight)
            })
            .collect();

        // Generate text by sampling weighted words
        let mut result = Vec::new();
        for _ in 0..word_count {
            let total_weight: f64 = weighted_words.iter().map(|(_, w)| w).sum();
            let mut roll = rng.gen_range(0.0..total_weight);
            let mut chosen = weighted_words[0].0;
            for (word, weight) in &weighted_words {
                roll -= weight;
                if roll <= 0.0 {
                    chosen = word;
                    break;
                }
            }
            result.push(chosen);
        }

        result.join(" ")
    }

    /// Get the current active alphabet.
    pub fn active_keys(&self) -> &[char] {
        &self.active_keys
    }

    /// Get mastered keys.
    pub fn mastered_keys(&self) -> &[char] {
        &self.mastered_keys
    }

    /// Get confidence for a specific key.
    pub fn key_confidence(&self, ch: char, stats: &TypingStats) -> f64 {
        if let Some(key_stats) = stats.key_stats.get(&ch) {
            let avg = key_stats.recent_avg_latency();
            if avg <= 0.0 || key_stats.hit_count < 3 {
                return 0.0;
            }
            let target_latency = 60000.0 / (self.target_wpm * 5.0);
            target_latency / avg
        } else {
            0.0
        }
    }
}
