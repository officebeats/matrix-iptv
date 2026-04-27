// Matrix Rain Animation for FTUE
use crate::app::{App, MatrixColumn};
use rand::Rng;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Clear, Paragraph, Wrap},
    Frame,
};

const MATRIX_CHARS: &[char] = &[
    // Safe ASCII / Width-1 characters to prevent layout breaks in Windows/Hyper
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I',
    'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', 'a', 'b',
    'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u',
    'v', 'w', 'x', 'y', 'z', '@', '#', '$', '%', '&', '*', '+', '=', '<', '>',
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

pub fn update_matrix_rain(
    columns: &mut [MatrixColumn],
    area: Rect,
    tick: u64,
    logo_hits: &mut [bool],
    show_logo: bool,
) {
    let mut rng = rand::thread_rng();

    let logo_width = 103;
    let logo_height = LOGO_LINES.len() as u16;
    let logo_x = area.x + area.width.saturating_sub(logo_width) / 2;
    let logo_y = area.y + area.height.saturating_sub(logo_height) / 2;

    for column in columns.iter_mut() {
        // Update position based on speed
        if tick.is_multiple_of(column.speed as u64) {
            column.y += 1;

            // Hit detection for building logo: Check the head AND the immediate trail segment
            if show_logo && column.x >= logo_x && column.x < logo_x + logo_width {
                let lx = (column.x - logo_x) as usize;
                // Check the head and a few pixels of the tail to "paint" the logo in more solidly
                for i in 0..6 {
                    // Check top 6 chars of the falling column
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
                if rng.gen_bool(0.05) {
                    // 5% chance per frame per column
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
    let buf = f.buffer_mut();

    // Fill background with black
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            if let Some(cell) = buf.cell_mut((x, y)) {
                cell.set_char(' ');
                cell.set_style(Style::default().bg(Color::Rgb(0, 0, 0)));
            }
        }
    }

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

                if gx >= area.left() && gx < area.right() && gy >= area.top() && gy < area.bottom()
                {
                    if let Some(line) = LOGO_LINES.get(ly as usize) {
                        if let Some(c) = line.chars().nth(lx as usize) {
                            if c != ' ' {
                                if let Some(cell) = buf.cell_mut((gx, gy)) {
                                    cell.set_char(c);
                                    cell.set_style(
                                        Style::default()
                                            .fg(crate::ui::colors::MATRIX_GREEN)
                                            .add_modifier(Modifier::BOLD),
                                    );
                                }
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
            let trace_style = Style::default().fg(Color::Rgb(0, 40, 0));
            for (lx, c) in line.chars().enumerate() {
                let gx = logo_x + lx as u16;
                if gx >= area.left() && gx < area.right() && c != ' ' {
                    if let Some(cell) = buf.cell_mut((gx, gy)) {
                        // Only draw ghosting if not already hit
                        let idx = ly * (logo_width as usize) + lx;
                        if app.matrix_rain_logo_hits.get(idx) != Some(&true) {
                            cell.set_char(c);
                            cell.set_style(trace_style);
                        }
                    }
                }
            }
        }
    }

    // 3. Render Matrix rain columns
    let mut rng = rand::thread_rng();
    for column in &app.matrix_rain_columns {
        let gx = area.x + column.x;
        if gx < area.left() || gx >= area.right() {
            continue;
        }

        for (i, &ch) in column.chars.iter().enumerate() {
            let y = column.y.saturating_sub(i as u16);
            let gy = area.y + y;

            if gy >= area.top() && gy < area.bottom() {
                // Check if this rain character overlaps with a logo character
                let mut logo_char = None;
                if gx >= logo_x
                    && gx < logo_x + logo_width
                    && gy >= logo_y
                    && gy < logo_y + logo_height
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

                let is_glitch = rng.gen_bool(0.01);
                let (draw_ch, style) = if let Some(lc) = logo_char {
                    let color = if i == 0 {
                        Color::White
                    } else if i < 15 {
                        Color::Rgb(180, 255, 180)
                    } else {
                        crate::ui::colors::MATRIX_GREEN
                    };
                    (lc, Style::default().fg(color).add_modifier(Modifier::BOLD))
                } else {
                    let color = if i == 0 || is_glitch {
                        Color::White
                    } else if i < 6 {
                        Color::Rgb(150, 255, 150)
                    } else {
                        crate::ui::colors::MATRIX_GREEN
                    };
                    (ch, Style::default().fg(color).add_modifier(Modifier::BOLD))
                };

                if let Some(cell) = buf.cell_mut((gx, gy)) {
                    cell.set_char(draw_ch);
                    cell.set_style(style);
                }
            }
        }
    }
}

pub fn render_matrix_edge_border(f: &mut Frame, area: Rect, margin_v: u16, margin_h: u16) {
    let buf = f.buffer_mut();
    #[cfg(not(target_arch = "wasm32"))]
    let ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    #[cfg(not(target_arch = "wasm32"))]
    let t = (ms % 1000000) as f32 / 1000.0 * 0.5;
    #[cfg(target_arch = "wasm32")]
    let t = ((js_sys::Date::now() as u64) % 1000000) as f32 / 1000.0 * 0.5;

    // Ordered by visual density (Variable Typographic ASCII)
    let char_set = [
        ' ', '.', '-', '~', ':', '=', '+', '*', 'x', '#', '%', 'W', '@', '█',
    ];

    let w = area.width as f32;
    let h = area.height as f32;

    // Define 3 slow-moving boundary attractors using intersecting sine waves
    let a1x = w * 0.5 + f32::sin(t * 0.8) * w * 0.4;
    let a1y = h * 0.5 + f32::cos(t * 0.9) * h * 0.4;

    let a2x = w * 0.5 + f32::cos(t * 1.2 + 1.0) * w * 0.3;
    let a2y = h * 0.5 + f32::sin(t * 1.5 + 2.0) * h * 0.4;

    let a3x = w * 0.5 + f32::sin(t * 0.5 + 2.0) * w * 0.45;
    let a3y = h * 0.5 + f32::sin(t * 0.7 + 1.0) * h * 0.35;

    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            let in_top = y < area.top() + margin_v;
            let in_bottom = y >= area.bottom() - margin_v;
            let in_left = x < area.left() + margin_h;
            let in_right = x >= area.right() - margin_h;

            if in_top || in_bottom || in_left || in_right {
                if let Some(cell) = buf.cell_mut((x, y)) {
                    let fx = x as f32;
                    let fy = y as f32 * 2.0; // Terminal chars are typically ~2x taller than wide

                    let py1 = a1y * 2.0;
                    let py2 = a2y * 2.0;
                    let py3 = a3y * 2.0;

                    // Calculate squared distances to attractors
                    let d1 = (fx - a1x).powi(2) + (fy - py1).powi(2) + 1.0;
                    let d2 = (fx - a2x).powi(2) + (fy - py2).powi(2) + 1.0;
                    let d3 = (fx - a3x).powi(2) + (fy - py3).powi(2) + 1.0;

                    // Convert distances into brightness blob intensities
                    // exp(-d/scale) creates a nice Gaussian metaball shape
                    let blob1 = f32::exp(-d1 / 600.0) * 1.2;
                    let blob2 = f32::exp(-d2 / 400.0) * 1.0;
                    let blob3 = f32::exp(-d3 / 700.0) * 1.5;

                    // Add a wavy low-frequency background noise layer
                    let wave = (f32::sin(fx * 0.08 + t) * f32::cos(fy * 0.05 - t) + 1.0) * 0.15;

                    let mut brightness = blob1 + blob2 + blob3 + wave;
                    brightness = brightness.clamp(0.0, 1.0);

                    // Map brightness to character index
                    let idx = (brightness * (char_set.len() - 1) as f32).round() as usize;
                    let ch = char_set[idx];

                    if ch != ' ' {
                        // Color styling based on brightness density gradient
                        let color = if brightness > 0.85 {
                            Color::White
                        } else if brightness > 0.65 {
                            Color::Rgb(150, 255, 150)
                        } else if brightness > 0.4 {
                            crate::ui::colors::MATRIX_GREEN
                        } else if brightness > 0.2 {
                            Color::Rgb(0, 100, 0)
                        } else {
                            Color::Rgb(0, 40, 0)
                        };

                        cell.set_char(ch);
                        cell.set_style(Style::default().fg(color).add_modifier(Modifier::BOLD));
                    } else {
                        cell.set_char(' ');
                        cell.set_style(Style::default().bg(Color::Rgb(0, 0, 0)));
                    }
                }
            }
        }
    }
}

pub fn render_welcome_popup(f: &mut Frame, app: &App, area: Rect) {
    let popup_area = centered_rect(70, 60, area);

    f.render_widget(Clear, popup_area);

    let inner = crate::ui::common::render_composite_block(
        f,
        popup_area,
        Some(" // SYSTEM_INITIALIZATION // "),
    );

    let mut text = vec![
        Line::from(""),
        Line::from(Span::styled(
            "WELCOME TO MATRIX IPTV",
            Style::default()
                .fg(crate::ui::colors::MATRIX_GREEN)
                .add_modifier(Modifier::BOLD),
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
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(vec![
                Span::styled("1. ", Style::default().fg(Color::White)),
                Span::styled("Press ", Style::default().fg(Color::White)),
                Span::styled(
                    "[n]",
                    Style::default()
                        .fg(crate::ui::colors::MATRIX_GREEN)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    " to add your first IPTV playlist",
                    Style::default().fg(Color::White),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("2. ", Style::default().fg(Color::White)),
                Span::styled(
                    "Enter your Xtream Codes credentials",
                    Style::default().fg(Color::White),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("3. ", Style::default().fg(Color::White)),
                Span::styled(
                    "Start watching Live TV, Movies, and Series!",
                    Style::default().fg(Color::White),
                ),
            ]),
            Line::from(""),
            Line::from(""),
        ]);
    }

    // Always show disclaimer
    text.extend(vec![
        Line::from(Span::styled(
            "⚠ DISCLAIMER:",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
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
            Style::default()
                .fg(crate::ui::colors::MATRIX_GREEN)
                .add_modifier(Modifier::ITALIC),
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
