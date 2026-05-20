# Typer

A cross-platform typing practice app built with Tauri (Rust + TypeScript).

## Features

- **Adaptive Learning** — Keys introduced gradually, starting with home row. New keys unlock when existing ones reach confidence threshold.
- **Sprint Detection** — Tracks typing bursts. Measures WPM per sprint with configurable idle timeout (default 3s).
- **Keyboard Heatmap** — Real-time visualization of key press frequency. Color intensity shows usage.
- **Per-Key Statistics** — Hit count, miss count, average latency, best latency, and confidence for every key.
- **Practice Text Generation** — Words weighted by weak keys. Struggling keys appear more often.
- **Dark Theme** — Easy on the eyes.

## Architecture

- **Rust backend** (Tauri) — Keyboard layout mapping, sprint detection engine, per-key statistics, adaptive lesson generator, IPC commands
- **TypeScript frontend** — Practice UI with text display, keyboard heatmap, stats dashboard, settings

## Building

```bash
# Install dependencies
npm install

# Development
npm run tauri dev

# Production build
npm run tauri build
```

### Prerequisites

- [Rust](https://rustup.rs/)
- [Node.js](https://nodejs.org/)
- Platform-specific Tauri dependencies:
  - **Linux**: `webkit2gtk-4.1`, `libappindicator3-dev`, `librsvg2-dev`
  - **macOS**: Xcode, CocoaPods
  - **Windows**: WebView2, Visual Studio Build Tools

## How It Works

### Adaptive Algorithm

Inspired by [keybr.com](https://keybr.com)'s guided learning approach:

1. Start with 6 home row keys (F, J, D, K, S, L)
2. Track typing speed (latency) and accuracy per key
3. Calculate confidence: `target_latency / actual_latency`
4. When ALL active keys reach confidence ≥ 1.0, introduce the next key
5. Practice text generation weights words containing weak keys higher

### Sprint Detection

A sprint is a burst of continuous typing:
- Starts with the first keypress after idle period
- Ends when no keypress for N seconds (default 3s, configurable)
- Each sprint calculates: WPM, accuracy, hits, misses
- Rolling average WPM across recent sprints

## License

MIT
