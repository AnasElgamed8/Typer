// Tauri invoke wrapper
const invoke = (window as any).__TAURI__.core.invoke as <T>(cmd: string, args?: Record<string, unknown>) => Promise<T>;

import { AvatarStateMachine } from './avatar';
import type { AvatarSet } from './avatar';
import { loadAllAvatarSets, getAvatarByName, BONGO_CAT_AVATAR } from './avatar-config';
import { getHand } from './keymap';

// ============================================================
// Types
// ============================================================

interface KeypressResult {
  correct: boolean;
  position: number;
  wpm: number;
  accuracy: number;
  completed_sprint: SprintInfo | null;
  heatmap: Record<string, number>;
  confidence: Record<string, number>;
}

interface SprintInfo {
  wpm: number;
  accuracy: number;
  hits: number;
  misses: number;
  duration_ms: number;
}

interface StatsResponse {
  total_hits: number;
  total_misses: number;
  accuracy: number;
  avg_wpm: number;
  sprint_count: number;
  active_keys: string[];
  mastered_keys: string[];
  key_details: KeyDetail[];
}

interface KeyDetail {
  character: string;
  hit_count: number;
  miss_count: number;
  avg_latency_ms: number;
  recent_avg_latency_ms: number;
  best_latency_ms: number;
  confidence: number;
}

// ============================================================
// State
// ============================================================

let currentText = "";
let currentPosition = 0;
let isTyping = false;
let charStates: ("pending" | "correct" | "incorrect" | "corrected")[] = [];
let avatar: AvatarStateMachine | null = null;
let allAvatarSets: AvatarSet[] = [];

// ============================================================
// DOM Elements
// ============================================================

const textDisplay = document.getElementById("textDisplay")!;
const wpmValue = document.getElementById("wpmValue")!;
const accuracyValue = document.getElementById("accuracyValue")!;
const sprintValue = document.getElementById("sprintValue")!;
const activeKeysValue = document.getElementById("activeKeysValue")!;
const keyboard = document.getElementById("keyboard")!;
const sprintNotification = document.getElementById("sprintNotification")!;

// ============================================================
// Keyboard Layout Definition
// ============================================================

interface KeyDef {
  label: string;
  width?: number;
  home?: boolean;
  id: string;
}

const KEYBOARD_ROWS: KeyDef[][] = [
  [
    { id: "\x60", label: "\x60" }, { id: "1", label: "1" }, { id: "2", label: "2" },
    { id: "3", label: "3" }, { id: "4", label: "4" }, { id: "5", label: "5" },
    { id: "6", label: "6" }, { id: "7", label: "7" }, { id: "8", label: "8" },
    { id: "9", label: "9" }, { id: "0", label: "0" }, { id: "-", label: "-" },
    { id: "=", label: "=" },
  ],
  [
    { id: "q", label: "Q" }, { id: "w", label: "W" }, { id: "e", label: "E" },
    { id: "r", label: "R" }, { id: "t", label: "T" }, { id: "y", label: "Y" },
    { id: "u", label: "U" }, { id: "i", label: "I" }, { id: "o", label: "O" },
    { id: "p", label: "P" }, { id: "[", label: "[" }, { id: "]", label: "]" },
    { id: "\\", label: "\\" },
  ],
  [
    { id: "a", label: "A", home: true }, { id: "s", label: "S", home: true },
    { id: "d", label: "D", home: true }, { id: "f", label: "F", home: true },
    { id: "g", label: "G" }, { id: "h", label: "H" },
    { id: "j", label: "J", home: true }, { id: "k", label: "K", home: true },
    { id: "l", label: "L", home: true }, { id: ";", label: ";", home: true },
    { id: "'", label: "'" },
  ],
  [
    { id: "z", label: "Z" }, { id: "x", label: "X" }, { id: "c", label: "C" },
    { id: "v", label: "V" }, { id: "b", label: "B" }, { id: "n", label: "N" },
    { id: "m", label: "M" }, { id: ",", label: "," }, { id: ".", label: "." },
    { id: "/", label: "/" },
  ],
  [
    { id: " ", label: "Space", width: 220 },
  ],
];

// ============================================================
// Render Functions
// ============================================================

function renderKeyboard() {
  keyboard.innerHTML = "";
  for (const row of KEYBOARD_ROWS) {
    const rowEl = document.createElement("div");
    rowEl.className = "keyboard-row";
    for (const keyDef of row) {
      const keyEl = document.createElement("div");
      keyEl.className = "key";
      if (keyDef.width) keyEl.style.width = keyDef.width + "px";
      if (keyDef.home) keyEl.classList.add("home");
      keyEl.id = "key-" + keyDef.id;

      const heatEl = document.createElement("div");
      heatEl.className = "key-heat";
      keyEl.appendChild(heatEl);

      const labelEl = document.createElement("span");
      labelEl.className = "key-label";
      labelEl.textContent = keyDef.label;
      keyEl.appendChild(labelEl);

      rowEl.appendChild(keyEl);
    }
    keyboard.appendChild(rowEl);
  }
}

function renderText(text: string, position: number) {
  textDisplay.innerHTML = "";
  charStates = new Array(text.length).fill("pending");
  for (let i = 0; i < text.length; i++) {
    const span = document.createElement("span");
    span.className = "char";
    span.textContent = text[i];

    if (i < position) {
      span.classList.add("correct");
      charStates[i] = "correct";
    } else if (i === position) {
      span.classList.add("current");
    } else {
      span.classList.add("pending");
    }

    textDisplay.appendChild(span);
  }
}

function markCharCorrect(position: number, correct: boolean) {
  const chars = textDisplay.querySelectorAll(".char");
  const idx = position - 1;
  if (idx >= 0 && idx < chars.length) {
    chars[idx].classList.remove("current");
    if (correct) {
      chars[idx].classList.add("correct");
      charStates[idx] = "correct";
    } else {
      chars[idx].classList.add("incorrect");
      charStates[idx] = "incorrect";
      setTimeout(() => chars[idx].classList.remove("incorrect"), 300);
    }
  }
  if (position < chars.length) {
    chars[position].classList.add("current");
  }
}

function flashKeyOnBoard(keyId: string) {
  const keyEl = document.getElementById("key-" + keyId);
  if (!keyEl) return;
  keyEl.classList.remove("flash");
  void keyEl.offsetWidth;
  keyEl.classList.add("flash");
  keyEl.addEventListener("animationend", () => {
    keyEl.classList.remove("flash");
  }, { once: true });
}

function updateHeatmap(heatmap: Record<string, number>, activeKeys: string[], masteredKeys: string[]) {
  let maxPresses = 0;
  for (const ch of activeKeys) {
    const count = heatmap[ch] || 0;
    if (count > maxPresses) maxPresses = count;
  }

  for (const keyDef of KEYBOARD_ROWS.flat()) {
    const keyEl = document.getElementById("key-" + keyDef.id);
    if (!keyEl) continue;

    const ch = keyDef.id;
    const count = heatmap[ch] || 0;
    const isActive = activeKeys.includes(ch);
    const isMastered = masteredKeys.includes(ch);

    keyEl.classList.toggle("active", isActive && count > 0);
    keyEl.classList.toggle("mastered", isMastered);

    const heatEl = keyEl.querySelector(".key-heat") as HTMLElement;
    if (heatEl && isActive && maxPresses > 0) {
      const ratio = count / maxPresses;
      const level = Math.min(9, Math.floor(ratio * 9));
      heatEl.className = "key-heat heat-" + level;
    }
  }
}

function showSprintNotification(sprint: SprintInfo) {
  const sprintWpm = document.getElementById("sprintWpm")!;
  const sprintAcc = document.getElementById("sprintAccuracy")!;
  const sprintHits = document.getElementById("sprintHits")!;

  sprintWpm.textContent = Math.round(sprint.wpm) + " WPM";
  sprintAcc.textContent = Math.round(sprint.accuracy * 100) + "%";
  sprintHits.textContent = sprint.hits + " hits";

  sprintNotification.style.display = "block";
  sprintNotification.style.animation = "none";
  void sprintNotification.offsetWidth;
  sprintNotification.style.animation = "";

  setTimeout(() => {
    sprintNotification.style.display = "none";
  }, 3500);
}

// ============================================================
// Avatar System
// ============================================================

function createAvatarContainer() {
  const container = document.createElement("div");
  container.id = "avatarContainer";
  container.className = "avatar-container";
  document.getElementById("app")!.appendChild(container);
  return container;
}

async function initAvatar() {
  const container = createAvatarContainer();

  // Load all available avatar sets (bundled + custom)
  const bundled = await loadAllAvatarSets();
  const custom = await loadCustomAvatars();
  allAvatarSets = [...bundled, ...custom];

  // Read saved avatar preference (default to Bongo Cat)
  const savedName = localStorage.getItem("typer-avatar") || BONGO_CAT_AVATAR.name;
  const selectedSet = getAvatarByName(allAvatarSets, savedName);

  // Create the state machine
  avatar = new AvatarStateMachine(container, selectedSet);

  // Populate the settings dropdown
  populateAvatarSelector();
}

function populateAvatarSelector() {
  const select = document.getElementById("avatarSelect") as HTMLSelectElement | null;
  if (!select) return;

  select.innerHTML = "";

  // "No avatar" option
  const noneOpt = document.createElement("option");
  noneOpt.value = "__none__";
  noneOpt.textContent = "No Avatar";
  select.appendChild(noneOpt);

  // Avatar sets
  for (const set of allAvatarSets) {
    const opt = document.createElement("option");
    opt.value = set.name;
    opt.textContent = set.name;
    select.appendChild(opt);
  }

  // Restore selection
  const savedName = localStorage.getItem("typer-avatar") || BONGO_CAT_AVATAR.name;
  if (savedName === "__none__") {
    select.value = "__none__";
  } else {
    select.value = savedName;
  }
}

function onAvatarSelectionChange(name: string) {
  if (!avatar) return;

  localStorage.setItem("typer-avatar", name);

  if (name === "__none__") {
    avatar.setEnabled(false);
  } else {
    const set = getAvatarByName(allAvatarSets, name);
    avatar.setAvatarSet(set);
    avatar.setEnabled(true);
  }
}

// ============================================================
// Custom Avatar Loading (via Tauri commands)
// ============================================================

interface CustomAvatarInfo {
  name: string;
  path: string;
  has_idle: boolean;
  has_left: boolean;
  has_right: boolean;
}

async function loadCustomAvatars(): Promise<AvatarSet[]> {
  try {
    const customAvatars = await invoke<CustomAvatarInfo[]>("list_custom_avatars");
    const sets: AvatarSet[] = [];

    for (const info of customAvatars) {
      if (info.has_idle && info.has_left && info.has_right) {
        // Use convertFileSrc for local file paths in Tauri v2
        const convertFileSrc = (window as any).__TAURI__.core.convertFileSrc as (path: string) => string;
        sets.push({
          name: info.name,
          idle: convertFileSrc(info.path + "/idle.png"),
          left: convertFileSrc(info.path + "/left.png"),
          right: convertFileSrc(info.path + "/right.png"),
        });
      }
    }

    return sets;
  } catch (_e) {
    // Custom avatars not available (e.g. running in dev without Tauri)
    return [];
  }
}

// ============================================================
// Tauri IPC
// ============================================================

async function loadNewLesson() {
  try {
    const text = await invoke<string>("get_lesson");
    currentText = text;
    currentPosition = 0;
    isTyping = false;
    renderText(text, 0);
  } catch (e) {
    console.error("Failed to load lesson:", e);
    textDisplay.textContent = "Error loading lesson: " + e;
  }
}

async function handleKeypress(key: string, shift: boolean) {
  if (!currentText || currentPosition >= currentText.length) return;

  const timestamp = performance.now();

  try {
    const result = await invoke<KeypressResult>("record_keypress", {
      key,
      shift,
      timestamp,
    });

    markCharCorrect(result.position, result.correct);
    currentPosition = result.position;

    flashKeyOnBoard(key.toLowerCase() === " " ? " " : key.toLowerCase());

    wpmValue.textContent = Math.round(result.wpm).toString();
    accuracyValue.textContent = Math.round(result.accuracy * 100) + "%";

    updateHeatmapFromResult(result);

    if (result.completed_sprint) {
      showSprintNotification(result.completed_sprint);
      sprintValue.textContent = (parseInt(sprintValue.textContent || "0") + 1).toString();
    }

    if (currentPosition >= currentText.length) {
      setTimeout(() => loadNewLesson(), 500);
    }
  } catch (e) {
    console.error("Keypress error:", e);
  }
}

async function updateHeatmapFromResult(result: KeypressResult) {
  try {
    const stats = await invoke<StatsResponse>("get_stats");
    updateHeatmap(result.heatmap, stats.active_keys, stats.mastered_keys);
    activeKeysValue.textContent = stats.active_keys.length.toString();
  } catch (_e) {
    const allKeys = Object.keys(result.heatmap);
    updateHeatmap(result.heatmap, allKeys, []);
  }
}

async function showStatsModal() {
  const modal = document.getElementById("statsModal")!;
  const content = document.getElementById("statsContent")!;

  try {
    const stats = await invoke<StatsResponse>("get_stats");

    content.innerHTML = `
      <div class="stats-summary">
        <div class="stat-card">
          <div class="value">${Math.round(stats.avg_wpm)}</div>
          <div class="label">Avg WPM</div>
        </div>
        <div class="stat-card">
          <div class="value">${Math.round(stats.accuracy * 100)}%</div>
          <div class="label">Accuracy</div>
        </div>
        <div class="stat-card">
          <div class="value">${stats.sprint_count}</div>
          <div class="label">Sprints</div>
        </div>
      </div>
      <p style="margin-bottom:12px;color:var(--text-secondary);font-size:13px;font-family:var(--font-mono);">
        Active: ${stats.active_keys.join(", ").toUpperCase() || "none"}<br>
        Mastered: ${stats.mastered_keys.join(", ").toUpperCase() || "none"}
      </p>
      ${stats.key_details.length > 0 ? `
        <table class="stats-table">
          <thead>
            <tr>
              <th>Key</th>
              <th>Hits</th>
              <th>Misses</th>
              <th>Avg Latency</th>
              <th>Confidence</th>
            </tr>
          </thead>
          <tbody>
            ${stats.key_details.map(k => `
              <tr>
                <td>${k.character === " " ? "Space" : k.character.toUpperCase()}</td>
                <td>${k.hit_count}</td>
                <td>${k.miss_count}</td>
                <td>${Math.round(k.avg_latency_ms)}ms</td>
                <td>${(k.confidence * 100).toFixed(0)}%</td>
              </tr>
            `).join("")}
          </tbody>
        </table>
      ` : '<p style="color:var(--text-dim)">No data yet. Start typing!</p>'}
    `;

    modal.style.display = "flex";
  } catch (e) {
    content.innerHTML = `<p>Error loading stats: ${e}</p>`;
    modal.style.display = "flex";
  }
}

// ============================================================
// Settings Modal
// ============================================================

function showSettingsModal() {
  const modal = document.getElementById("settingsModal")!;
  modal.style.display = "flex";
}

function hideSettingsModal() {
  const modal = document.getElementById("settingsModal")!;
  modal.style.display = "none";
}

async function saveSettings() {
  const idleTimeout = parseFloat(
    (document.getElementById("idleTimeoutInput") as HTMLInputElement).value
  ) * 1000;
  const targetWpm = parseFloat(
    (document.getElementById("targetWpmInput") as HTMLInputElement).value
  );
  const wordCount = parseInt(
    (document.getElementById("wordCountInput") as HTMLInputElement).value
  );

  // Save avatar selection
  const avatarSelect = document.getElementById("avatarSelect") as HTMLSelectElement;
  if (avatarSelect) {
    onAvatarSelectionChange(avatarSelect.value);
  }

  try {
    await invoke("update_settings", {
      idleTimeoutMs: idleTimeout,
      targetWpm,
      wordCount,
    });
    hideSettingsModal();
    await loadNewLesson();
  } catch (e) {
    console.error("Failed to save settings:", e);
  }
}

// ============================================================
// Event Listeners
// ============================================================

document.addEventListener("keydown", (e: KeyboardEvent) => {
  if (["Shift", "Control", "Alt", "Meta", "CapsLock", "Tab", "Escape"].includes(e.key)) {
    return;
  }

  if (e.key === "Enter") {
    e.preventDefault();
    return;
  }

  if (isTyping && (e.ctrlKey || e.metaKey)) {
    return;
  }

  e.preventDefault();

  if (!isTyping) {
    isTyping = true;
  }

  // Update avatar state
  if (avatar) {
    avatar.onKeyDown(getHand(e.key));
  }

  handleKeypress(e.key, e.shiftKey);
});

document.addEventListener("keyup", (e: KeyboardEvent) => {
  if (["Shift", "Control", "Alt", "Meta", "CapsLock", "Tab", "Escape"].includes(e.key)) {
    return;
  }

  // Return avatar to idle after key release
  if (avatar) {
    avatar.onKeyUp();
  }
});

document.getElementById("newLessonBtn")!.addEventListener("click", loadNewLesson);
document.getElementById("statsBtn")!.addEventListener("click", showStatsModal);
document.getElementById("settingsBtn")!.addEventListener("click", showSettingsModal);
document.getElementById("closeSettingsBtn")!.addEventListener("click", hideSettingsModal);
document.getElementById("cancelSettingsBtn")!.addEventListener("click", hideSettingsModal);
document.getElementById("saveSettingsBtn")!.addEventListener("click", saveSettings);
document.getElementById("closeStatsBtn")!.addEventListener("click", () => {
  document.getElementById("statsModal")!.style.display = "none";
});
document.getElementById("closeStatsBtn2")!.addEventListener("click", () => {
  document.getElementById("statsModal")!.style.display = "none";
});
document.getElementById("resetStatsBtn")!.addEventListener("click", async () => {
  if (confirm("Reset all statistics? This cannot be undone.")) {
    await invoke("reset_stats");
    document.getElementById("statsModal")!.style.display = "none";
    sprintValue.textContent = "0";
    await loadNewLesson();
  }
});

document.querySelectorAll(".modal-overlay").forEach(overlay => {
  overlay.addEventListener("click", (e) => {
    if (e.target === overlay) {
      (overlay as HTMLElement).style.display = "none";
    }
  });
});

// ============================================================
// Init
// ============================================================

renderKeyboard();
loadNewLesson();
initAvatar();
