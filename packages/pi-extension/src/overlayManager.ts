import type { OverlayHandle } from "@earendil-works/pi-tui";

/**
 * Tracks all open overlay handles so Boss-key hide/restore covers every overlay
 * automatically. Any overlay that registers itself here is guaranteed to receive
 * setHidden() calls — no manual bookkeeping required when adding new overlays.
 */
export class OverlayManager {
  private handles = new Set<OverlayHandle>();

  /** Register a freshly opened overlay and apply the current hidden state. */
  register(handle: OverlayHandle, hidden: boolean): void {
    this.handles.add(handle);
    handle.setHidden(hidden);
  }

  /** Unregister a closing overlay. */
  unregister(handle: OverlayHandle): void {
    this.handles.delete(handle);
  }

  /** Apply hidden state to all registered overlays (Boss-key toggle). */
  setAllHidden(hidden: boolean): void {
    this.handles.forEach((h) => h.setHidden(hidden));
  }

  /** Returns true if any overlay is currently open. */
  hasAny(): boolean {
    return this.handles.size > 0;
  }
}
