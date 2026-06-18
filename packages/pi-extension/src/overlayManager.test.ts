import { describe, expect, it, vi } from "vitest";
import type { OverlayHandle } from "@earendil-works/pi-tui";
import { OverlayManager } from "./overlayManager.ts";

function makeHandle(): OverlayHandle & { setHidden: ReturnType<typeof vi.fn> } {
  return { setHidden: vi.fn(), hide: vi.fn() } as unknown as OverlayHandle & {
    setHidden: ReturnType<typeof vi.fn>;
  };
}

describe("OverlayManager", () => {
  it("starts empty", () => {
    const mgr = new OverlayManager();
    expect(mgr.hasAny()).toBe(false);
  });

  it("register applies current hidden state immediately", () => {
    const mgr = new OverlayManager();
    const h = makeHandle();
    mgr.register(h, true);
    expect(h.setHidden).toHaveBeenCalledWith(true);
    expect(mgr.hasAny()).toBe(true);
  });

  it("register with hidden=false does not hide", () => {
    const mgr = new OverlayManager();
    const h = makeHandle();
    mgr.register(h, false);
    expect(h.setHidden).toHaveBeenCalledWith(false);
  });

  it("unregister removes handle from tracking", () => {
    const mgr = new OverlayManager();
    const h = makeHandle();
    mgr.register(h, false);
    mgr.unregister(h);
    expect(mgr.hasAny()).toBe(false);
  });

  it("setAllHidden propagates to every registered handle", () => {
    const mgr = new OverlayManager();
    const h1 = makeHandle();
    const h2 = makeHandle();
    mgr.register(h1, false);
    mgr.register(h2, false);
    mgr.setAllHidden(true);
    expect(h1.setHidden).toHaveBeenLastCalledWith(true);
    expect(h2.setHidden).toHaveBeenLastCalledWith(true);
  });

  it("setAllHidden does not affect unregistered handles", () => {
    const mgr = new OverlayManager();
    const h = makeHandle();
    mgr.register(h, false);
    mgr.unregister(h);
    h.setHidden.mockClear();
    mgr.setAllHidden(true);
    expect(h.setHidden).not.toHaveBeenCalled();
  });

  it("hasAny returns false after all handles are unregistered", () => {
    const mgr = new OverlayManager();
    const h1 = makeHandle();
    const h2 = makeHandle();
    mgr.register(h1, false);
    mgr.register(h2, false);
    mgr.unregister(h1);
    mgr.unregister(h2);
    expect(mgr.hasAny()).toBe(false);
  });
});
