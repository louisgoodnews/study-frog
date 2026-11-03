# Appearance and start-up settings

Written on 2026-11-03 (TASK-0221) after the settings were built (the owner asked to "just go ahead and add all
settings"). They are stored per user and profile in `app.toml`, in an `[appearance]` table, and only values that differ
from the defaults are written. Every setting takes effect at once and is saved at once; none needs a restart.

| Key | Values | Default | Where in the app |
|---|---|---|---|
| `theme` | `system`, `dark`, `light` | `system` | Settings > Appearance; the profile menu |
| `zoom` | 50 to 300 (percent); the app offers 70, 80, 90, 100, 110, 125, 150, 175, 200 | 100 | Settings > Appearance; Ctrl and plus, Ctrl and minus, Ctrl and 0 |
| `text_size` | 70 to 200 (percent); offered 85, 90, 100, 110, 125, 150 | 100 | Settings > Appearance |
| `density` | `compact`, `comfortable`, `spacious` | `comfortable` | Settings > Appearance |
| `accent` | `blue`, `green`, `purple`, `orange`, `pink`, `teal` | `blue` | Settings > Appearance |
| `high_contrast` | `true` or `false` | `false` | Settings > Appearance |
| `reduced_motion` | `true` or `false` | `false` | Settings > Appearance |
| `start_screen` | `home`, `library`, `last` | `home` | Settings > Appearance |
| `last_screen` | a screen word (`calendar`, `board`, ...) | none | kept by the app while `start_screen = "last"` |
| `sidebar_folded` | `true` or `false` | `false` | Settings > Appearance |
| `remember_window` | `true` or `false` | `true` | Settings > Appearance |
| `window_width`, `window_height`, `window_x`, `window_y`, `window_maximized` | whole numbers, `true` | none | kept by the app while `remember_window` is on |

A wrong value is an error that names the key and the line, and the rest of `app.toml` is not used until it is fixed (as
for the other keys); unknown keys in the table are errors too.

## How each one works
- **Theme.** The first start follows the system's. `system` follows a change of the system's theme while the application
  runs; `dark` and `light` do not. Both palettes are data in `theme.rs`.
- **Interface size** is egui's zoom factor: everything gets larger or smaller, including the title bar. **Text size** only
  scales the text styles, on top of that. The keyboard shortcuts are ours, so the value is saved (egui's own are off).
- **Density** is one table of spacing: item spacing, button padding and the height of a control.
- **Accent** replaces the colour of selections, links, the main button and bars in both themes; each pair of accent and
  text colour on it meets 4.5 to 1. **High contrast** gives each theme text at 7 to 1 and strong borders; a test checks
  the ratios of every palette (`tests/appearance.rs`).
- **Reduced motion** sets egui's animation time to zero. Turning it off brings the default back; nothing else touches the
  animation time.
- **Start screen.** A vault opens on Home, the Library or the screen that was open last. A screen is remembered only
  while `last` is chosen (each change of screen then writes the file).
- **Sidebar folded** is how the sidebar starts; folding it with the button during a session is not saved.
- **Window.** The size, the place (when the system tells it: some Wayland desktops do not) and whether the window was
  maximised are saved once the window has stayed as it is for a second. A maximised window keeps the size and place it
  had before. At start-up the saved window and the saved look are used before the first frame.

## Not in this version
A language setting, custom fonts, a custom accent colour, and a different theme per vault. Colours in views come from
the palette (`tests/no_hardcoded_colours.rs` fails otherwise); the contrast of every palette is tested. Screens were
looked at in both themes with `studyfrog --screenshot FILE --screen NAME --theme dark|light`; the Create dialog, the
import and export dialog and the PDF viewer were not.
