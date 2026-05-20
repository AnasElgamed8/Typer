use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Physical key code mapped to a position on the keyboard.
/// We use scan codes for cross-platform consistency.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PhysicalKey {
    /// Row on the keyboard (0 = number row, 1 = QWERTY row, etc.)
    pub row: u8,
    /// Column on the keyboard (0 = leftmost)
    pub col: u8,
}

impl PhysicalKey {
    pub fn new(row: u8, col: u8) -> Self {
        Self { row, col }
    }
}

/// Finger assignment for a physical key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Finger {
    LeftPinky,
    LeftRing,
    LeftMiddle,
    LeftIndex,
    LeftThumb,
    RightThumb,
    RightIndex,
    RightMiddle,
    RightRing,
    RightPinky,
}

/// Maps a character to its physical position and finger assignment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyMapping {
    pub physical: PhysicalKey,
    pub finger: Finger,
    pub label: String,
    pub is_home: bool,
}

/// QWERTY layout definition.
/// Maps lowercase characters to their physical positions.
pub fn qwerty_layout() -> HashMap<char, KeyMapping> {
    let mut map = HashMap::new();

    // Helper closure
    let mut add = |ch: char, row: u8, col: u8, finger: Finger, label: &str, is_home: bool| {
        map.insert(ch, KeyMapping {
            physical: PhysicalKey::new(row, col),
            finger,
            label: label.to_string(),
            is_home,
        });
    };

    // Number row
    add('`', 0, 0, Finger::LeftPinky, "`", false);
    add('1', 0, 1, Finger::LeftPinky, "1", false);
    add('2', 0, 2, Finger::LeftRing, "2", false);
    add('3', 0, 3, Finger::LeftMiddle, "3", false);
    add('4', 0, 4, Finger::LeftIndex, "4", false);
    add('5', 0, 5, Finger::LeftIndex, "5", false);
    add('6', 0, 6, Finger::RightIndex, "6", false);
    add('7', 0, 7, Finger::RightIndex, "7", false);
    add('8', 0, 8, Finger::RightMiddle, "8", false);
    add('9', 0, 9, Finger::RightRing, "9", false);
    add('0', 0, 10, Finger::RightPinky, "0", false);
    add('-', 0, 11, Finger::RightPinky, "-", false);
    add('=', 0, 12, Finger::RightPinky, "=", false);

    // QWERTY row
    add('q', 1, 0, Finger::LeftPinky, "Q", false);
    add('w', 1, 1, Finger::LeftRing, "W", false);
    add('e', 1, 2, Finger::LeftMiddle, "E", false);
    add('r', 1, 3, Finger::LeftIndex, "R", false);
    add('t', 1, 4, Finger::LeftIndex, "T", false);
    add('y', 1, 5, Finger::RightIndex, "Y", false);
    add('u', 1, 6, Finger::RightIndex, "U", false);
    add('i', 1, 7, Finger::RightMiddle, "I", false);
    add('o', 1, 8, Finger::RightRing, "O", false);
    add('p', 1, 9, Finger::RightPinky, "P", false);
    add('[', 1, 10, Finger::RightPinky, "[", false);
    add(']', 1, 11, Finger::RightPinky, "]", false);
    add('\\', 1, 12, Finger::RightPinky, "\\", false);

    // Home row
    add('a', 2, 0, Finger::LeftPinky, "A", true);
    add('s', 2, 1, Finger::LeftRing, "S", true);
    add('d', 2, 2, Finger::LeftMiddle, "D", true);
    add('f', 2, 3, Finger::LeftIndex, "F", true);
    add('g', 2, 4, Finger::LeftIndex, "G", false);
    add('h', 2, 5, Finger::RightIndex, "H", false);
    add('j', 2, 6, Finger::RightIndex, "J", true);
    add('k', 2, 7, Finger::RightMiddle, "K", true);
    add('l', 2, 8, Finger::RightRing, "L", true);
    add(';', 2, 9, Finger::RightPinky, ";", true);
    add('\'', 2, 10, Finger::RightPinky, "'", false);

    // Bottom row
    add('z', 3, 0, Finger::LeftPinky, "Z", false);
    add('x', 3, 1, Finger::LeftRing, "X", false);
    add('c', 3, 2, Finger::LeftMiddle, "C", false);
    add('v', 3, 3, Finger::LeftIndex, "V", false);
    add('b', 3, 4, Finger::LeftIndex, "B", false);
    add('n', 3, 5, Finger::RightIndex, "N", false);
    add('m', 3, 6, Finger::RightIndex, "M", false);
    add(',', 3, 7, Finger::RightMiddle, ",", false);
    add('.', 3, 8, Finger::RightRing, ".", false);
    add('/', 3, 9, Finger::RightPinky, "/", false);

    // Space bar
    add(' ', 4, 5, Finger::RightThumb, "Space", false);

    map
}

/// Map a JavaScript KeyboardEvent.key to our internal character.
/// Returns None for modifier keys, function keys, etc.
pub fn js_key_to_char(key: &str, shift: bool) -> Option<char> {
    if key.len() == 1 {
        let ch = key.chars().next().unwrap();
        if shift {
            // Map shifted characters
            match ch {
                'a'..='z' => Some(ch.to_ascii_uppercase()),
                '1' => Some('!'),
                '2' => Some('@'),
                '3' => Some('#'),
                '4' => Some('$'),
                '5' => Some('%'),
                '6' => Some('^'),
                '7' => Some('&'),
                '8' => Some('*'),
                '9' => Some('('),
                '0' => Some(')'),
                '-' => Some('_'),
                '=' => Some('+'),
                '[' => Some('{'),
                ']' => Some('}'),
                '\\' => Some('|'),
                ';' => Some(':'),
                '\'' => Some('"'),
                ',' => Some('<'),
                '.' => Some('>'),
                '/' => Some('?'),
                '`' => Some('~'),
                _ => Some(ch),
            }
        } else {
            Some(ch.to_ascii_lowercase())
        }
    } else {
        match key {
            "Space" => Some(' '),
            _ => None,
        }
    }
}

/// Is this a printable character we track?
pub fn is_printable(ch: char) -> bool {
    ch.is_ascii_graphic() || ch == ' '
}
