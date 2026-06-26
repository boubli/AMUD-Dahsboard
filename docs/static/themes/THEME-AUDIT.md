# AMUD Theme Compatibility Audit

Audit date: 2026-06-25. Covers default + 18 bundled themes after guest-compact and filled-metrics layout.

| Theme | Guest compact | Filled metrics | Wallpaper | Settings page | Light mode | Notes |
|-------|---------------|----------------|-----------|---------------|------------|-------|
| AMUD Default | pass | pass | pass | pass | pass | Base `style.css` |
| dracula | pass | pass | pass | pass | warn | Dark-only; light scheduler may strip body bg |
| nord | pass | pass | pass | pass | warn | |
| cyberpunk-neon | pass | pass | pass | pass | warn | Animated accents guarded by theme-guards |
| sunset-warm | pass | pass | pass | pass | warn | |
| catppuccin-mocha | pass | pass | pass | pass | warn | |
| gruvbox-dark | pass | pass | pass | pass | warn | |
| tokyo-night | pass | pass | pass | pass | warn | |
| one-dark | pass | pass | pass | pass | warn | |
| everforest | pass | pass | pass | pass | warn | |
| monokai | pass | pass | pass | pass | warn | |
| rose-pine | pass | pass | pass | pass | warn | |
| solarized-dark | pass | pass | pass | pass | warn | |
| terminal-phosphor | pass | pass | n/a | pass | warn | Self-contained CRT; no wallpaper |
| vaporwave-grid | pass | pass | n/a | pass | warn | Self-contained grid |
| blueprint-tech | pass | pass | n/a | pass | warn | Self-contained blueprint |
| luxury-gold | pass | pass | pass | pass | warn | Wallpaper layered |
| holographic-prism | pass | pass | pass | pass | warn | Local ::before guard duplicated in theme-guards |
| brutalist-mono | pass | pass | n/a | pass | warn | Hides wallpaper layers |

## Global fixes applied

- `theme-guards.css`: guest-compact overrides theme min-height and grid span
- `theme-guards.css`: disables `.app-card.glass-panel::before` for metric stability
- Appearance UI: warning when light mode + custom CSS active

## Bundled themes are dark-oriented

Light mode (`data-theme="light"`) in `style.css` clears body backgrounds. Use dark mode with custom themes for best results.
