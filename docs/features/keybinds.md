# Keybinds

All keyboard shortcuts can be changed in Preferences → Keybinds.

## Default bindings

| Action | Default |
|---|---|
| New Project | Ctrl+N |
| Open Project | Ctrl+O |
| Save Project | Ctrl+S |
| Export | Ctrl+P |
| Animation Preview | P |

## Changing a binding

Open Preferences (Edit → Preferences), switch to the Keybinds tab, then click **Change** next to the action you want to rebind. The row shows "Press any key..." — press the new key combination. Modifier keys (Ctrl, Alt, Shift) are recorded as part of the binding. Press **Escape** to cancel without changing anything.

Click **Reset to defaults** to restore all five bindings to the values above.

## Supported keys

Letters A–Z, digits 0–9, function keys F1–F12, and the special keys Enter, Space, Tab, Delete, Backspace, Home, End, and Insert. Any combination of Ctrl, Alt, and Shift can be added as modifiers.

## Technical Notes

Bindings are stored in the `[keybinds]` section of `prefs.toml` (located in the platform config directory, e.g. `~/.config/FastPack/prefs.toml` on Linux). Each entry stores the key name and three boolean modifier flags. Missing entries fall back to the defaults listed above, so removing the `[keybinds]` section resets everything.

While the Keybinds tab is open and capture mode is active (after clicking Change), the existing shortcuts are suspended so the key press is recorded rather than acted on.
