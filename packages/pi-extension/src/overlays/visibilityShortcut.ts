import type { KeyId } from "@earendil-works/pi-tui";
import { matchesKey } from "@earendil-works/pi-tui";

export type VisibilityShortcutOptions = {
  visibilityShortcut?: KeyId;
  onToggleVisibility?: () => Promise<void> | void;
};

export function handleVisibilityShortcut(keyData: string, options: VisibilityShortcutOptions): boolean {
  if (!options.visibilityShortcut || !options.onToggleVisibility) return false;
  if (!matchesKey(keyData, options.visibilityShortcut)) return false;
  void options.onToggleVisibility();
  return true;
}
