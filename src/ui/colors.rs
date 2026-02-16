use ratatui::style::Color;

// ─── Matrix Terminal Theme (Claude Code / Gemini CLI Inspired) ───
// Monochromatic: Green + White + Grays. No gray-on-black (unreadable).
// Every "dim" color must be visible on pure black terminals.

// Primary accent — the signature matrix green
pub const MATRIX_GREEN: Color = Color::Rgb(0, 255, 65);

// Softer green for borders, secondary elements (visible but not blinding)
pub const SOFT_GREEN: Color = Color::Rgb(0, 200, 50);

// Dim green for inactive borders, separators (MUST be visible on black)
pub const DARK_GREEN: Color = Color::Rgb(0, 120, 30);

// Subtle background tint for selected rows (not full inversion)
pub const HIGHLIGHT_BG: Color = Color::Rgb(0, 40, 10);

// Text hierarchy — all must be clearly visible on pure black
pub const BRIGHT_GREEN: Color = Color::White;     // Primary text (white for max contrast)
pub const TEXT_PRIMARY: Color = Color::White;      // Main content text
pub const TEXT_SECONDARY: Color = Color::Rgb(200, 200, 200); // Secondary labels — bright enough on black
pub const TEXT_DIM: Color = Color::Rgb(180, 180, 180); // Dimmer text — still clearly white-ish
pub const TEXT_MUTED: Color = Color::Rgb(140, 140, 140);  // Structural separators — visible

// Status colors (restrained — only for meaning)
pub const STATUS_LIVE: Color = Color::Rgb(255, 100, 100);
pub const STATUS_ENDED: Color = Color::Rgb(160, 160, 160);
pub const STATUS_WARN: Color = Color::Rgb(255, 200, 80);

// Legacy mappings (backward compat)
pub const MODERN_BG: Color = Color::Black;
pub const MODERN_FG: Color = TEXT_PRIMARY;
pub const MODERN_GRAY: Color = TEXT_DIM;
pub const MODERN_BLUE: Color = MATRIX_GREEN;
pub const MODERN_CYAN: Color = MATRIX_GREEN;
pub const MODERN_PURPLE: Color = TEXT_PRIMARY;
pub const MODERN_GREEN: Color = MATRIX_GREEN;
pub const MODERN_YELLOW: Color = TEXT_PRIMARY;
pub const MODERN_RED: Color = Color::Rgb(255, 0, 0);
pub const BRIGHT_YELLOW: Color = TEXT_PRIMARY;
pub const BRIGHT_GRAY: Color = TEXT_SECONDARY;
pub const CP_GREEN: Color = MATRIX_GREEN;
pub const CP_CYAN: Color = MATRIX_GREEN;
pub const CP_YELLOW: Color = TEXT_PRIMARY;
pub const CP_WHITE: Color = TEXT_PRIMARY;
pub const CP_GRAY: Color = TEXT_SECONDARY;
