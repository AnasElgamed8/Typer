// ============================================================
// Avatar Config — Loading & Managing Avatar Sets
// ============================================================

import type { AvatarSet } from './avatar';

// Bundled avatar: Bongo Cat (SVG assets in src/assets/)
import bongoIdle from './assets/avatars/bongo-cat/idle.svg';
import bongoLeft from './assets/avatars/bongo-cat/left.svg';
import bongoRight from './assets/avatars/bongo-cat/right.svg';

/** The default Bongo Cat avatar set (bundled). */
export const BONGO_CAT_AVATAR: AvatarSet = {
  name: 'Bongo Cat',
  idle: bongoIdle,
  left: bongoLeft,
  right: bongoRight,
};

/** All built-in avatar sets. */
export const BUNDLED_AVATARS: AvatarSet[] = [BONGO_CAT_AVATAR];

/** Shape of the custom avatar info returned from the Rust backend. */
interface CustomAvatarInfo {
  name: string;
  path: string;
  has_idle: boolean;
  has_left: boolean;
  has_right: boolean;
}

// Tauri invoke wrapper
const invoke = (window as any).__TAURI__?.core?.invoke as
  (<T>(cmd: string, args?: Record<string, unknown>) => Promise<T>) | undefined;

/**
 * Load all avatar sets — bundled defaults + custom user avatars from
 * ~/.config/typer/avatars/{name}/{idle,left,right}.{png|svg}
 */
export async function loadAllAvatarSets(): Promise<AvatarSet[]> {
  const sets: AvatarSet[] = [...BUNDLED_AVATARS];

  // Load custom avatars via Tauri command
  if (invoke) {
    try {
      const convertFileSrc = (window as any).__TAURI__?.core?.convertFileSrc as
        ((path: string) => string) | undefined;

      const customAvatars = await invoke<CustomAvatarInfo[]>('list_custom_avatars');

      for (const info of customAvatars) {
        if (info.has_idle && info.has_left && info.has_right) {
          // Use convertFileSrc to create asset URLs from local file paths
          const toUrl = convertFileSrc
            ? (p: string) => convertFileSrc(p)
            : (p: string) => `asset://localhost/${p}`;

          sets.push({
            name: info.name,
            idle: toUrl(`${info.path}/idle.png`),
            left: toUrl(`${info.path}/left.png`),
            right: toUrl(`${info.path}/right.png`),
          });
        }
      }
    } catch (_e) {
      // Custom avatars not available (e.g. running outside Tauri)
      // Silently fall back to bundled only
    }
  }

  return sets;
}

/**
 * Get avatar set by name from a list.
 * Returns the first set if name not found.
 */
export function getAvatarByName(sets: AvatarSet[], name: string): AvatarSet {
  return sets.find(s => s.name === name) || sets[0];
}
