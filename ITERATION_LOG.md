# Typer — Iteration Log

> Append-only log of all work done on the Typer project.
> Each process logs here. Final reviews are sent to Anas's Telegram.

---

## Research Phase — 2026-05-21 00:17

- Cloned empty repo from https://github.com/AnasElgamed8/Typer.git
- Studied keybr.com's codebase (aradzie/keybr.com, 4398★, AGPL-3.0)
- **Key findings:**
  - Adaptive lesson: minimum 6 keys, new key when all active keys reach confidence ≥ 1.0
  - Confidence = targetSpeed / actualSpeed (or target_latency / actual_latency)
  - Learning rate: polynomial regression on last 30 samples
  - Text generation: phonetic model for realistic word combinations
  - Session detection: >1hr gap OR performance degradation
  - License: AGPL-3.0 — CANNOT copy code, can only study algorithm
- **Decision:** Build our own implementation inspired by keybr's algorithm, not a fork

## Iteration 1 Build — 2026-05-21 00:20

- Set up Tauri v2 project (vanilla-ts template)
- Installed Rust toolchain (rustc 1.95.0)
- **Rust backend modules:**
  - `keyboard.rs` — QWERTY layout definition, key mapping, js_key_to_char
  - `stats.rs` — Per-key statistics (hits, misses, latency tracking, rolling averages)
  - `sprint.rs` — Sprint detection with configurable idle timeout
  - `adaptive.rs` — Adaptive lesson generator (keybr-inspired algorithm)
  - `lib.rs` — Tauri IPC commands (record_keypress, get_lesson, get_stats, update_settings, reset_stats)
- **Frontend:**
  - `index.html` — Practice UI with text display, stats bar, keyboard heatmap, modals
  - `styles.css` — Dark theme (deep blue palette), keyboard heatmap colors (9 levels)
  - `main.ts` — Keyboard event handling, Tauri IPC, text rendering, heatmap updates
- **Features implemented:**
  - ✅ Adaptive learning (home row first, confidence-based key introduction)
  - ✅ Sprint detection (configurable idle timeout, default 3s)
  - ✅ Per-key statistics (hits, misses, latency, confidence)
  - ✅ Keyboard heatmap (9-level color intensity)
  - ✅ Practice text generation (weighted by weak keys)
  - ✅ Settings modal (idle timeout, target WPM, words per lesson)
  - ✅ Stats modal (per-key table, summary cards)
  - ✅ Dark theme
  - ✅ Sprint notifications (slide-in card with WPM/accuracy)
- **Cannot test in container** (no webkit2gtk) — needs to be built on Anas's Arch machine

## Iteration 1 Review — 2026-05-21 00:35

**Verdict: NEEDS TESTING — code is complete but unverified**

### Code Quality Assessment

| Dimension | Score | Notes |
|-----------|-------|-------|
| Rust Backend | 8/10 | Clean modules, proper types, serde serialization. Sprint detector has edge case: first keypress in sprint has 0 latency. |
| Frontend | 7/10 | Dark theme, keyboard heatmap, modals. Missing: no loading states, no error boundaries. |
| Algorithm | 8/10 | Faithful adaptation of keybr's approach. Confidence formula correct. Word weighting by latency is smart. |
| IPC Design | 8/10 | Clean command signatures. KeypressResult bundles everything the frontend needs. |
| Type Safety | 6/10 | Frontend uses `window.__TAURI__` directly instead of proper imports. No TypeScript types for Tauri API. |
| Error Handling | 5/10 | Rust side: unwrap() on mutex locks (will panic on poison). Frontside: minimal error display. |

### Critical Issues
1. **Untested** — Cannot build in container (no webkit2gtk). Must test on actual machine.
2. **Mutex poisoning** — All `.lock().unwrap()` calls will panic if a thread panics while holding the lock. Should use `.lock().unwrap_or_else(|e| e.into_inner())`.
3. **No `modules/mod.rs`** — The modules directory has a mod.rs but lib.rs declares `mod keyboard; mod stats;` etc. at the root level, not through modules/. Need to verify Rust compilation.

### What Works (Theoretically)
- Adaptive algorithm introduces keys in the right order (home → top → bottom)
- Sprint detection with idle timeout
- Per-key latency tracking with rolling averages
- Keyboard heatmap with 9 color levels
- Settings persistence during session
- Stats modal with per-key breakdown

### What Needs Fixing
1. Rust compilation (verify module structure)
2. TypeScript types for Tauri API
3. Error handling (don't panic on mutex poison)
4. First keypress latency edge case in sprint detector
5. Test everything end-to-end

### Score: Incomplete (untested)
Cannot assign a final score until it's built and tested. The code LOOKS correct but the traffic monitor also looked correct and didn't count bytes.

---

## Next Steps
- Push to GitHub
- Anas builds on his machine (`npm install && npm run tauri dev`)
- Report actual behavior: does typing work? Does the heatmap update? Does sprint detection trigger?
- Fix any issues found in testing
- Then iterate: avatar system, gaming mode, more languages
