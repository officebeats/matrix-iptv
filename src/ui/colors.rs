use ratatui::style::Color;

// Cyberpunk Theme Palette (Optimized for Visibility)
pub const CP_GREEN: Color = Color::Rgb(57, 255, 20);   // Vibrant Neon Green
// pub const CP_PINK: Color = Color::Rgb(255, 105, 180);  // Hot Pink
pub const CP_CYAN: Color = Color::Rgb(0, 255, 255);    // Bright Cyan (Light Blue)
pub const CP_YELLOW: Color = Color::Rgb(255, 255, 0);  // Pure Yellow
pub const CP_WHITE: Color = Color::White;              // Pure White
pub const CP_GRAY: Color = Color::Rgb(220, 220, 220);  // Bright Silver (for unselected)

// Matrix Palette
pub const MATRIX_GREEN: Color = Color::Rgb(0, 255, 65);   // Classic Matrix Neon
pub const DARK_GREEN: Color = Color::Rgb(0, 100, 0);     // Deep Terminal Green
pub const BRIGHT_GREEN: Color = Color::Rgb(150, 255, 150); // Lighter Neon highlight
pub const BRIGHT_YELLOW: Color = CP_YELLOW; 
pub const BRIGHT_GRAY: Color = CP_GRAY;
