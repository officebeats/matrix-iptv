use ratatui::style::Color;

// Cyberpunk Theme Palette (Optimized for Visibility)
pub const CP_GREEN: Color = Color::Rgb(57, 255, 20);   // Vibrant Neon Green
// pub const CP_PINK: Color = Color::Rgb(255, 105, 180);  // Hot Pink
pub const CP_CYAN: Color = Color::Rgb(0, 255, 255);    // Bright Cyan (Light Blue)
pub const CP_YELLOW: Color = Color::Rgb(255, 255, 0);  // Pure Yellow
pub const CP_WHITE: Color = Color::White;              // Pure White
pub const CP_GRAY: Color = Color::Rgb(220, 220, 220);  // Bright Silver (for unselected)

// Mappings
pub const MATRIX_GREEN: Color = CP_GREEN;
pub const DARK_GREEN: Color = CP_GREEN; // Use Green for borders too for that Matrix vibe
pub const BRIGHT_GREEN: Color = CP_CYAN; // Use Cyan for highlights now instead of Yellow for variety
pub const BRIGHT_YELLOW: Color = CP_YELLOW; 
// pub const BRIGHT_WHITE: Color = Color::White;
pub const BRIGHT_GRAY: Color = CP_GRAY;
