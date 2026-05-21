// ============================================================
// Avatar State Machine
// ============================================================

import { type Hand } from './keymap';

export type AvatarState = 'idle' | 'left' | 'right';

export interface AvatarSet {
  name: string;
  idle: string;   // URL/path to idle image
  left: string;   // URL/path to left-paw-down image
  right: string;  // URL/path to right-paw-down image
}

export class AvatarStateMachine {
  private state: AvatarState = 'idle';
  private container: HTMLElement;
  private images: Map<AvatarState, HTMLImageElement> = new Map();
  private holdTimer: ReturnType<typeof setTimeout> | null = null;
  private idleTimer: ReturnType<typeof setTimeout> | null = null;
  private pawHoldTime: number; // ms to hold paw after keyup
  private enabled: boolean = true;

  constructor(
    container: HTMLElement,
    private avatarSet: AvatarSet,
    pawHoldTime: number = 150,
  ) {
    this.container = container;
    this.pawHoldTime = pawHoldTime;
    this.loadImages();
  }

  /** Pre-load all images to avoid flicker on first keystroke. */
  private loadImages() {
    const states: AvatarState[] = ['idle', 'left', 'right'];
    for (const state of states) {
      const img = document.createElement('img');
      img.src = this.avatarSet[state];
      img.alt = `Avatar ${state}`;
      img.className = 'avatar-image';
      img.draggable = false;
      if (state !== 'idle') {
        img.style.opacity = '0';
      }
      this.container.appendChild(img);
      this.images.set(state, img);
    }
    // Add idle breathing animation class
    this.images.get('idle')?.classList.add('avatar-idle-breathe');
  }

  /** Swap the avatar set (e.g. when user selects a different one). */
  setAvatarSet(newSet: AvatarSet) {
    this.avatarSet = newSet;
    // Update image sources
    const states: AvatarState[] = ['idle', 'left', 'right'];
    for (const state of states) {
      const img = this.images.get(state);
      if (img) {
        img.src = newSet[state];
      }
    }
  }

  /** Enable or disable the avatar. */
  setEnabled(enabled: boolean) {
    this.enabled = enabled;
    this.container.style.display = enabled ? '' : 'none';
  }

  /** Handle a keypress — switch avatar state based on which hand. */
  onKeyDown(hand: Hand) {
    if (!this.enabled) return;

    if (this.holdTimer) {
      clearTimeout(this.holdTimer);
      this.holdTimer = null;
    }

    const newState: AvatarState = hand === 'space' ? 'left' : hand;

    // Only switch if state actually changes
    if (newState !== this.state) {
      this.transitionTo(newState);
    }
  }

  /** Handle keyup — hold for a short time then return to idle. */
  onKeyUp() {
    if (!this.enabled) return;

    if (this.pawHoldTime > 0) {
      this.holdTimer = setTimeout(() => {
        this.transitionTo('idle');
      }, this.pawHoldTime);
    } else {
      this.transitionTo('idle');
    }
  }

  private transitionTo(newState: AvatarState) {
    const oldImg = this.images.get(this.state);
    const newImg = this.images.get(newState);

    if (oldImg) {
      oldImg.style.opacity = '0';
      if (this.state === 'idle') {
        oldImg.classList.remove('avatar-idle-breathe');
      }
    }
    if (newImg) {
      newImg.style.opacity = '1';
      if (newState === 'idle') {
        newImg.classList.add('avatar-idle-breathe');
      }
    }

    this.state = newState;
  }

  /** Get the current state (for debugging). */
  getState(): AvatarState {
    return this.state;
  }

  destroy() {
    if (this.holdTimer) clearTimeout(this.holdTimer);
    if (this.idleTimer) clearTimeout(this.idleTimer);
    this.container.innerHTML = '';
  }
}
