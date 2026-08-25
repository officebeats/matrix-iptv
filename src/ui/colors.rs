use ratatui::style::Color;

// ─── Theme tokens (OMP oh-my-pi treatment) ───
// Token structure mirrors OMP's theme system: accent / border / muted / dim /
// text hierarchy, with selection expressed as accent-colored cursor + text
// instead of background fills. Matrix green stays the brand accent.

// Primary accent — the signature matrix green (OMP: `accent`)
pub const ACCENT: Color = Color::Rgb(0, 255, 65);

// Quiet chrome — borders and separators (OMP: `border` / `borderMuted`)
// Titanium's subtleGray, slightly green-shifted to keep the identity.
pub const BORDER: Color = Color::Rgb(42, 56, 46);
pub const BORDER_MUTED: Color = Color::Rgb(31, 37, 45);

// Text hierarchy (OMP: `text` > `muted` > `dim`)
pub const TEXT_PRIMARY: Color = Color::White;
pub const TEXT_MUTED: Color = Color::Rgb(156, 163, 176); // dimAluminum
pub const TEXT_DIM: Color = Color::Rgb(107, 114, 128); // OMP `dim`
pub const TEXT_SECONDARY: Color = TEXT_MUTED;

// Semantic status (OMP: success / error / warning)
pub const SUCCESS: Color = Color::Rgb(0, 255, 136); // readoutGreen
pub const ERROR_C: Color = Color::Rgb(255, 71, 87); // alertRed
pub const WARNING: Color = Color::Rgb(255, 179, 71); // warningAmber

// Selection — OMP uses accent fg + cursor glyph for keyboard selection;
// selectedBg is reserved for mouse-hover bands.
pub const SELECTED_BG: Color = Color::Rgb(0, 40, 10);

// ─── Legacy aliases (existing call sites; values re-pointed to tokens) ───
pub const MATRIX_GREEN: Color = ACCENT;
pub const SOFT_GREEN: Color = BORDER;
pub const DARK_GREEN: Color = BORDER_MUTED;
pub const HIGHLIGHT_BG: Color = SELECTED_BG;
pub const BRIGHT_GREEN: Color = TEXT_PRIMARY;

// Status colors (legacy names)
pub const STATUS_LIVE: Color = ERROR_C;
pub const STATUS_ENDED: Color = Color::Rgb(160, 160, 160);
pub const STATUS_WARN: Color = WARNING;

// Legacy mappings (backward compat)
pub const MODERN_BG: Color = Color::Rgb(0, 0, 0);
pub const MODERN_FG: Color = TEXT_PRIMARY;
pub const MODERN_GRAY: Color = TEXT_DIM;
pub const MODERN_BLUE: Color = ACCENT;
pub const MODERN_CYAN: Color = ACCENT;
pub const MODERN_PURPLE: Color = TEXT_PRIMARY;
pub const MODERN_GREEN: Color = ACCENT;
pub const MODERN_YELLOW: Color = TEXT_PRIMARY;
pub const MODERN_RED: Color = ERROR_C;
pub const BRIGHT_YELLOW: Color = TEXT_PRIMARY;
pub const BRIGHT_GRAY: Color = TEXT_SECONDARY;
pub const CP_GREEN: Color = ACCENT;
pub const CP_CYAN: Color = ACCENT;
pub const CP_YELLOW: Color = TEXT_PRIMARY;
pub const CP_WHITE: Color = TEXT_PRIMARY;
pub const CP_GRAY: Color = TEXT_SECONDARY;

// ─── OMP selection treatment ───
// Keyboard selection = accent cursor glyph + accent text, no background fill
// (OMP SelectList: `selectedText = theme.fg("accent")`, cursor `❯`).
pub const CURSOR_GLYPH: &str = "❯";

/// Highlight style for the selected list/table row: accent fg + bold, no bg.
pub fn selection_style() -> ratatui::style::Style {
    ratatui::style::Style::default()
        .fg(ACCENT)
        .add_modifier(ratatui::style::Modifier::BOLD)
}

/// Highlight symbol: accent-colored cursor + space (rendered via List's
/// highlight_symbol; color applied through selection_style on the row).
pub fn selection_symbol() -> String {
    format!("{} ", CURSOR_GLYPH)
}
