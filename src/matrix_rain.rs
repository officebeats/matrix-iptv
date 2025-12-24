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
    // Half-width Katakana (classic Matrix look)
    'ｦ', 'ｧ', 'ｨ', 'ｩ', 'ｪ', 'ｫ', 'ｬ', 'ｭ', 'ｮ', 'ｯ',
    'ｰ', 'ｱ', 'ｲ', 'ｳ', 'ｴ', 'ｵ', 'ｶ', 'ｷ', 'ｸ', 'ｹ',
    'ｺ', 'ｻ', 'ｼ', 'ｽ', 'ｾ', 'ｿ', 'ﾀ', 'ﾁ', 'ﾂ', 'ﾃ',
    'ﾄ', 'ﾅ', 'ﾆ', 'ﾇ', 'ﾈ', 'ﾉ', 'ﾊ', 'ﾋ', 'ﾌ', 'ﾍ',
    'ﾎ', 'ﾏ', 'ﾐ', 'ﾑ', 'ﾒ', 'ﾓ', 'ﾔ', 'ﾕ', 'ﾖ', 'ﾗ',
    'ﾘ', 'ﾙ', 'ﾚ', 'ﾛ', 'ﾜ', 'ﾝ',
    // Some numbers for variety
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
];

pub fn init_matrix_rain(area: Rect) -> Vec<MatrixColumn> {
    let mut rng = rand::thread_rng();
    let mut columns = Vec::new();
    
    // Create columns across the width - MORE RAIN!
    for x in 0..area.width {
        if rng.gen_bool(0.7) { // 70% chance for each column (was 30%)
            let length = rng.gen_range(8..30); // Longer columns (was 5..20)
            let speed = 1; // Faster speed - 10% faster (was 1..3, now always 1)
            let y = rng.gen_range(0..area.height);
            
            let mut chars = Vec::new();
            for _ in 0..length {
                chars.push(MATRIX_CHARS[rng.gen_range(0..MATRIX_CHARS.len())]);
            }
            
            columns.push(MatrixColumn {
                x,
                y,
                length,
                speed,
                chars,
            });
        }
    }
    
    columns
}

pub fn update_matrix_rain(columns: &mut Vec<MatrixColumn>, area: Rect, tick: u64) {
    let mut rng = rand::thread_rng();
    
    for column in columns.iter_mut() {
        // Update position based on speed
        if tick % column.speed as u64 == 0 {
            column.y += 1;
            
            // Reset column if it goes off screen
            if column.y > area.height + column.length {
                column.y = 0;
                column.x = rng.gen_range(0..area.width);
                
                // Randomize chars
                for i in 0..column.chars.len() {
                    column.chars[i] = MATRIX_CHARS[rng.gen_range(0..MATRIX_CHARS.len())];
                }
            }
        }
    }
}

pub fn render_matrix_rain(f: &mut Frame, app: &App, area: Rect) {
    // Matrix green color to match home screen
    const MATRIX_GREEN: Color = Color::Rgb(0, 255, 70);
    
    // ASCII Logo for "MATRIX IPTV CLI"
    let logo_lines = vec![
        "███╗   ███╗ █████╗ ████████╗██████╗ ██╗██╗  ██╗    ██╗██████╗ ████████╗██╗   ██╗     ██████╗██╗     ██╗",
        "████╗ ████║██╔══██╗╚══██╔══╝██╔══██╗██║╚██╗██╔╝    ██║██╔══██╗╚══██╔══╝██║   ██║    ██╔════╝██║     ██║",
        "██╔████╔██║███████║   ██║   ██████╔╝██║ ╚███╔╝     ██║██████╔╝   ██║   ██║   ██║    ██║     ██║     ██║",
        "██║╚██╔╝██║██╔══██║   ██║   ██╔══██╗██║ ██╔██╗     ██║██╔═══╝    ██║   ╚██╗ ██╔╝    ██║     ██║     ██║",
        "██║ ╚═╝ ██║██║  ██║   ██║   ██║  ██║██║██╔╝ ██╗    ██║██║        ██║    ╚████╔╝     ╚██████╗███████╗██║",
        "╚═╝     ╚═╝╚═╝  ╚═╝   ╚═╝   ╚═╝  ╚═╝╚═╝╚═╝  ╚═╝    ╚═╝╚═╝        ╚═╝     ╚═══╝       ╚═════╝╚══════╝╚═╝",
    ];
    
    // Calculate reveal progress based on elapsed time (0.0 to 1.0)
    let elapsed = if let Some(start) = app.matrix_rain_start_time {
        start.elapsed().as_millis() as f32 / 3000.0 // 3 seconds total
    } else {
        0.0
    };
    let reveal_progress = elapsed.min(1.0);
    
    // Render Matrix rain columns (background)
    for column in &app.matrix_rain_columns {
        for (i, &ch) in column.chars.iter().enumerate() {
            let y = column.y.saturating_sub(i as u16);
            
            if y < area.height && column.x < area.width {
                let brightness = if i == 0 {
                    Color::White // Head of the column is brightest
                } else if i < 3 {
                    Color::Rgb(0, 255, 0) // Bright green
                } else {
                    Color::Rgb(0, (200 - (i * 20).min(180)) as u8, 0) // Fading green
                };
                
                let cell_area = Rect {
                    x: area.x + column.x,
                    y: area.y + y,
                    width: 1,
                    height: 1,
                };
                
                let span = Span::styled(
                    ch.to_string(),
                    Style::default().fg(brightness).add_modifier(Modifier::BOLD),
                );
                
                f.render_widget(Paragraph::new(span), cell_area);
            }
        }
    }
    
    // Only show logo in startup mode (not screensaver mode)
    if !app.matrix_rain_screensaver_mode {
        // Logo appears immediately and stays for full 3 seconds
        if reveal_progress < 1.0 {
            // Calculate logo area (centered)
            let logo_height = logo_lines.len() as u16;
            let logo_y = (area.height / 2).saturating_sub(logo_height / 2);
            
            let logo_area = Rect {
                x: area.x,
                y: area.y + logo_y,
                width: area.width,
                height: logo_height.min(area.height.saturating_sub(logo_y)),
            };
            
            let logo_text = logo_lines.join("\n");
            let logo_widget = Paragraph::new(logo_text)
                .style(Style::default()
                    .fg(MATRIX_GREEN)
                    .add_modifier(Modifier::BOLD))
                .alignment(Alignment::Center);
            
            f.render_widget(logo_widget, logo_area);
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
        .border_style(Style::default().fg(Color::Rgb(0, 255, 0)).add_modifier(Modifier::BOLD));
    
    f.render_widget(block.clone(), popup_area);
    
    let inner = block.inner(popup_area);
    
    let mut text = vec![
        Line::from(""),
        Line::from(Span::styled(
            "WELCOME TO MATRIX IPTV",
            Style::default().fg(Color::Rgb(0, 255, 0)).add_modifier(Modifier::BOLD),
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
                Span::styled("[n]", Style::default().fg(Color::Rgb(0, 255, 0)).add_modifier(Modifier::BOLD)),
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
            Style::default().fg(Color::Rgb(0, 200, 0)).add_modifier(Modifier::ITALIC),
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
