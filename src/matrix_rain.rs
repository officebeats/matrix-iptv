// Matrix Rain Animation for FTUE
use crate::app::{App, MatrixColumn};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap},
    Frame,
};
use rand::Rng;

const MATRIX_CHARS: &[char] = &[
    // Mixed set: Katakana, Numbers, Roman, Symbols
    'ｦ', 'ｧ', 'ｨ', 'ｩ', 'ｪ', 'ｫ', 'ｬ', 'ｭ', 'ｮ', 'ｯ',
    'ｰ', 'ｱ', 'ｲ', 'ｳ', 'ｴ', 'ｵ', 'ｶ', 'ｷ', 'ｸ', 'ｹ',
    'ｺ', 'ｻ', 'ｼ', 'ｽ', 'ｾ', 'ｿ', 'ﾀ', 'ﾁ', 'ﾂ', 'ﾃ',
    'ﾄ', 'ﾅ', 'ﾆ', 'ﾇ', 'ﾈ', 'ﾉ', 'ﾊ', 'ﾋ', 'ﾌ', 'ﾍ',
    'ﾎ', 'ﾏ', 'ﾐ', 'ﾑ', 'ﾒ', 'ﾓ', 'ﾔ', 'ﾕ', 'ﾖ', 'ﾗ',
    'ﾘ', 'ﾙ', 'ﾚ', 'ﾛ', 'ﾜ', 'ﾝ',
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
    'A', 'B', 'C', 'D', 'E', 'F', 'Z', 'M', 'X', 'Q',
    'Ω', 'π', 'Ψ', 'Δ', '⚡',
];

pub fn init_matrix_rain(area: Rect) -> Vec<MatrixColumn> {
    let mut rng = rand::thread_rng();
    let mut columns = Vec::new();
    
    // Logo boundaries for guaranteed density
    let logo_width = 103;
    let logo_x_start = area.width.saturating_sub(logo_width) / 2;
    let logo_x_end = logo_x_start + logo_width;

    // Create columns across the width - ULTRA DENSE RAIN
    for x in 0..area.width {
        let is_logo_x = x >= logo_x_start && x < logo_x_end;
        // Boost density in logo area (multiple streams per column)
        let streams = if is_logo_x { 2 } else { 1 };
        
        for _ in 0..streams {
            // Halved density: 0.90 -> 0.45
            // But KEEP logo area guaranteed (is_logo_x) so the logo still builds properly
            if is_logo_x || rng.gen_bool(0.45) { 
                let length = rng.gen_range(10..40); // varied length
                let speed = rng.gen_range(1..4); // Varied speed (1=fastest, 3=slower)
                // Offset Y start to stagger particles in double streams
                let y = rng.gen_range(0..area.height + length);
                
                let mut chars = Vec::new();
                for _ in 0..length {
                    chars.push(MATRIX_CHARS[rng.gen_range(0..MATRIX_CHARS.len())]);
                }
                
                columns.push(MatrixColumn {
                    x,
                    y: y.saturating_sub(length), // Start offscreen or staggered
                    length,
                    speed,
                    chars,
                });
            }
        }
    }
    
    columns
}

pub fn update_matrix_rain(columns: &mut Vec<MatrixColumn>, area: Rect, tick: u64, logo_hits: &mut Vec<bool>, show_logo: bool) {
    let mut rng = rand::thread_rng();
    
    let logo_width = 103;
    let logo_height = LOGO_LINES.len() as u16;
    let logo_x = area.x + area.width.saturating_sub(logo_width) / 2;
    let logo_y = area.y + area.height.saturating_sub(logo_height) / 2;

    for column in columns.iter_mut() {
        // Update position based on speed
        if tick % column.speed as u64 == 0 {
            column.y += 1;

            // Hit detection for building logo: Check the head AND the immediate trail segment
            if show_logo {
                if column.x >= logo_x && column.x < logo_x + logo_width {
                    let lx = (column.x - logo_x) as usize;
                    // Check the head and a few pixels of the tail to "paint" the logo in more solidly
                    for i in 0..6 { // Check top 6 chars of the falling column
                        let py = column.y.saturating_sub(i);
                        if py >= logo_y && py < logo_y + logo_height {
                            let ly = (py - logo_y) as usize;
                            if let Some(line) = LOGO_LINES.get(ly) {
                                if let Some(c) = line.chars().nth(lx) {
                                    if c != ' ' {
                                        let idx = ly * (logo_width as usize) + lx;
                                        if idx < logo_hits.len() {
                                            logo_hits[idx] = true;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            
            // Reset column if it goes off screen
            if column.y > area.height + column.length {
                column.y = 0;
                column.x = rng.gen_range(0..area.width);
                
                // Randomize chars
                for i in 0..column.chars.len() {
                    column.chars[i] = MATRIX_CHARS[rng.gen_range(0..MATRIX_CHARS.len())];
                }
            } else {
                // Occasional "decryption" shift: change a random character in the column
                if rng.gen_bool(0.05) { // 5% chance per frame per column
                     let idx = rng.gen_range(0..column.chars.len());
                     column.chars[idx] = MATRIX_CHARS[rng.gen_range(0..MATRIX_CHARS.len())];
                }
            }
        }
    }
}

const LOGO_LINES: &[&str] = &[
    "███╗   ███╗ █████╗ ████████╗██████╗ ██╗██╗  ██╗    ██╗██████╗ ████████╗██╗   ██╗     ██████╗██╗     ██╗",
    "████╗ ████║██╔══██╗╚══██╔══╝██╔══██╗██║╚██╗██╔╝    ██║██╔══██╗╚══██╔══╝██║   ██║    ██╔════╝██║     ██║",
    "██╔████╔██║███████║   ██║   ██████╔╝██║ ╚███╔╝     ██║██████╔╝   ██║   ██║   ██║    ██║     ██║     ██║",
    "██║╚██╔╝██║██╔══██║   ██║   ██╔══██╗██║ ██╔██╗     ██║██╔═══╝    ██║   ╚██╗ ██╔╝    ██║     ██║     ██║",
    "██║ ╚═╝ ██║██║  ██║   ██║   ██║  ██║██║██╔╝ ██╗    ██║██║        ██║    ╚████╔╝     ╚██████╗███████╗██║",
    "╚═╝     ╚═╝╚═╝  ╚═╝   ╚═╝   ╚═╝  ╚═╝╚═╝╚═╝  ╚═╝    ╚═╝╚═╝        ╚═╝     ╚═══╝       ╚═════╝╚══════╝╚═╝",
];

pub fn render_matrix_rain(f: &mut Frame, app: &App, area: Rect) {
    // Clear background to hide UI completely (Startup and Screensaver)
    f.render_widget(Clear, area);
    let block = Block::default().style(Style::default().bg(Color::Black));
    f.render_widget(block, area);

    // Calculate logo position for hit detection
    let logo_width = 103;
    let logo_height = LOGO_LINES.len() as u16;
    let logo_x = area.x + area.width.saturating_sub(logo_width) / 2;
    let logo_y = area.y + area.height.saturating_sub(logo_height) / 2;
    
    // 1. Draw the "trace" (activated logo pixels)
    // This builds up the static logo as rain passes through it
    for ly in 0..logo_height {
        for lx in 0..logo_width {
            let idx = (ly as usize) * (logo_width as usize) + (lx as usize);
            if let Some(true) = app.matrix_rain_logo_hits.get(idx) {
                let gx = logo_x + lx;
                let gy = logo_y + ly;
                
                // CRITICAL FIX: Robust bounds check for the specific cell
                if gx >= area.left() && gx < area.right() && gy >= area.top() && gy < area.bottom() {
                    if let Some(line) = LOGO_LINES.get(ly as usize) {
                        if let Some(c) = line.chars().nth(lx as usize) {
                            if c != ' ' {
                                // Super bright neon green for activated pixels
                                let style = Style::default().fg(crate::ui::colors::MATRIX_GREEN).add_modifier(Modifier::BOLD);
                                f.render_widget(Paragraph::new(c.to_string()).style(style), Rect::new(gx, gy, 1, 1));
                            }
                        }
                    }
                }
            }
        }
    }

    // 2. Draw a VERY dim version of the remaining logo (ghosting)
    for (ly, line) in LOGO_LINES.iter().enumerate() {
        let gy = logo_y + ly as u16;
        if gy >= area.top() && gy < area.bottom() {
            // Slightly brighter ghost so it's easier to see structure early on
            let trace_style = Style::default().fg(Color::Rgb(0, 40, 0)); 
            let span = Span::styled(*line, trace_style);
            
            // CRITICAL FIX: Clip the logo width to the terminal width to prevent out-of-bounds panics
            let logo_area = Rect::new(logo_x, gy, logo_width, 1);
            let clipped_area = area.intersection(logo_area);
            
            if clipped_area.width > 0 {
                f.render_widget(
                    Paragraph::new(vec![Line::from(span)]),
                    clipped_area
                );
            }
        }
    }

    // Render Matrix rain columns
    for column in &app.matrix_rain_columns {
        for (i, &ch) in column.chars.iter().enumerate() {
            let y = column.y.saturating_sub(i as u16);
            
            let gx = area.x + column.x;
            let gy = area.y + y;

            // CRITICAL FIX: Ensure coordinate is within the rendered area
            if gx >= area.left() && gx < area.right() && gy >= area.top() && gy < area.bottom() {
                // Check if this rain character overlaps with a non-empty pixel of our logo
                let mut logo_char = None;
                if gx >= logo_x && gx < logo_x + logo_width &&
                   gy >= logo_y && gy < logo_y + logo_height
                {
                    let lx = (gx - logo_x) as usize;
                    let ly = (gy - logo_y) as usize;
                    if let Some(line) = LOGO_LINES.get(ly) {
                        if let Some(c) = line.chars().nth(lx) {
                            if c != ' ' {
                                logo_char = Some(c);
                            }
                        }
                    }
                }

                // ... rest of rendering logic ...
                // Occasional random bright glitch
                let is_glitch = rand::thread_rng().gen_bool(0.01);
                
                let (draw_ch, style) = if let Some(lc) = logo_char {
                    // Logo pixels "ignite" - use the logo character itself and make it very bright
                    let color = if i == 0 {
                        Color::White 
                    } else if i < 15 { // Longer highlight tail for logo
                        Color::Rgb(180, 255, 180)
                    } else {
                        crate::ui::colors::MATRIX_GREEN
                    };
                    (lc.to_string(), Style::default().fg(color).add_modifier(Modifier::BOLD))
                } else {
                    let color = if i == 0 || is_glitch {
                        Color::White // Head or glitch
                    } else if i < 6 {
                        Color::Rgb(150, 255, 150) // Bright white-green top
                    } else {
                        crate::ui::colors::MATRIX_GREEN // Deep green body
                    };
                    (ch.to_string(), Style::default().fg(color).add_modifier(Modifier::BOLD))
                };
                
                f.render_widget(Paragraph::new(Span::styled(draw_ch, style)), Rect::new(gx, gy, 1, 1));
            }
        }
    }
}

pub fn render_welcome_popup(f: &mut Frame, app: &App, area: Rect) {
    let popup_area = centered_rect(70, 60, area);
    
    f.render_widget(Clear, popup_area);
    
    let block = Block::default()
        .title(" // SYSTEM_INITIALIZATION // ")
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(crate::ui::colors::MATRIX_GREEN).add_modifier(Modifier::BOLD));
    
    f.render_widget(block.clone(), popup_area);
    
    let inner = block.inner(popup_area);
    
    let mut text = vec![
        Line::from(""),
        Line::from(Span::styled(
            "WELCOME TO MATRIX IPTV",
            Style::default().fg(crate::ui::colors::MATRIX_GREEN).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "A high-performance terminal IPTV player",
            Style::default().fg(Color::White),
        )),
        Line::from(""),
        Line::from(""),
    ];
    
    // Only show "GETTING STARTED" if user has no playlists
    if app.config.accounts.is_empty() {
        text.extend(vec![
            Line::from(Span::styled(
                "⚠ GETTING STARTED:",
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(vec![
                Span::styled("1. ", Style::default().fg(Color::White)),
                Span::styled("Press ", Style::default().fg(Color::White)),
                Span::styled("[n]", Style::default().fg(crate::ui::colors::MATRIX_GREEN).add_modifier(Modifier::BOLD)),
                Span::styled(" to add your first IPTV playlist", Style::default().fg(Color::White)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("2. ", Style::default().fg(Color::White)),
                Span::styled("Enter your Xtream Codes credentials", Style::default().fg(Color::White)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("3. ", Style::default().fg(Color::White)),
                Span::styled("Start watching Live TV, Movies, and Series!", Style::default().fg(Color::White)),
            ]),
            Line::from(""),
            Line::from(""),
        ]);
    }
    
    // Always show disclaimer
    text.extend(vec![
        Line::from(Span::styled(
            "⚠ DISCLAIMER:",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "This app does not provide any content. You must have",
            Style::default().fg(Color::Yellow),
        )),
        Line::from(Span::styled(
            "a valid IPTV subscription from a provider.",
            Style::default().fg(Color::Yellow),
        )),
        Line::from(""),
        Line::from(""),
        Line::from(Span::styled(
            "Press any key to continue...",
            Style::default().fg(crate::ui::colors::MATRIX_GREEN).add_modifier(Modifier::ITALIC),
        )),
    ]);
    
    let paragraph = Paragraph::new(text)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    
    f.render_widget(paragraph, inner);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
