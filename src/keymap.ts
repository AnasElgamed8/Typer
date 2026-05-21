// ============================================================
// Key → Hand Mapping for Avatar State Machine
// ============================================================

const LEFT_HAND_KEYS = new Set([
  '`', '1', '2', '3', '4', '5',
  'q', 'w', 'e', 'r', 't',
  'a', 's', 'd', 'f', 'g',
  'z', 'x', 'c', 'v', 'b',
]);

const RIGHT_HAND_KEYS = new Set([
  '6', '7', '8', '9', '0', '-', '=',
  'y', 'u', 'i', 'o', 'p', '[', ']', '\\',
  'h', 'j', 'k', 'l', ';', '\'',
  'n', 'm', ',', '.', '/',
]);

export type Hand = 'left' | 'right' | 'space';

/**
 * Determine which hand should press a given key on a QWERTY layout.
 * Returns 'space' for the spacebar (special "both paws" state).
 */
export function getHand(key: string): Hand {
  const k = key.toLowerCase();
  if (k === ' ' || k === 'space') return 'space';
  if (LEFT_HAND_KEYS.has(k)) return 'left';
  if (RIGHT_HAND_KEYS.has(k)) return 'right';
  // Default to right for unknown keys (Backspace, Enter, etc.)
  return 'right';
}
