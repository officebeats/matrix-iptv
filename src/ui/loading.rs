use crate::app::App;
use crate::ui::colors::{MATRIX_GREEN, SOFT_GREEN, TEXT_DIM, TEXT_PRIMARY, TEXT_SECONDARY};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

struct LoadingPanel {
    title: String,
    headline: String,
    detail: String,
    raw_status: String,
    percent: Option<usize>,
    counts: Option<(usize, usize)>,
    eta: Option<String>,
}

pub fn render_loading(f: &mut Frame, app: &App, area: Rect) {
    if !app.session.state_loading {
        return;
    }

    // Subtle overlay across the background
    let row = "░".repeat(area.width as usize);
    let lines = vec![Line::from(row.as_str()); area.height as usize];
    let dim_paragraph =
        Paragraph::new(lines).style(Style::default().fg(Color::DarkGray).bg(Color::Rgb(0, 0, 0)));
    f.render_widget(dim_paragraph, area);

    let panel = derive_loading_panel(app);

    let popup_area = centered_rect(84, 12, area);
    f.render_widget(Clear, popup_area);

    use ratatui::symbols::border;
    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(border::ROUNDED)
        .border_style(Style::default().fg(MATRIX_GREEN))
        .title(Span::styled(
            panel.title.clone(),
            Style::default()
                .fg(MATRIX_GREEN)
                .add_modifier(Modifier::BOLD),
        ))
        .title_alignment(Alignment::Center)
        .style(Style::default().bg(Color::Rgb(0, 0, 0)));

    f.render_widget(block, popup_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(2),
            Constraint::Length(1),
            Constraint::Length(2),
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(popup_area);

    let tick = app.session.loading_tick;

    // Matrix style Katakana spinner and decoding effect
    let katakana = [
        'ｦ', 'ｧ', 'ｨ', 'ｩ', 'ｪ', 'ｫ', 'ｬ', 'ｭ', 'ｮ', 'ｯ', 'ｰ', 'ｱ', 'ｲ', 'ｳ', 'ｴ', 'ｵ', 'ｶ', 'ｷ',
        'ｸ', 'ｹ', 'ｺ', 'ｻ', 'ｼ', 'ｽ', 'ｾ', 'ｿ', 'ﾀ', 'ﾁ', 'ﾂ', 'ﾃ', 'ﾄ', 'ﾅ', 'ﾆ', 'ﾇ', 'ﾈ', 'ﾉ',
        'ﾊ', 'ﾋ', 'ﾌ', 'ﾍ', 'ﾎ', 'ﾏ', 'ﾐ', 'ﾑ', 'ﾒ', 'ﾓ', 'ﾔ', 'ﾕ', 'ﾖ', 'ﾗ', 'ﾘ', 'ﾙ', 'ﾚ', 'ﾛ',
        'ﾜ', 'ﾝ',
    ];
    let spinner = katakana[(tick as usize) % katakana.len()];

    let glitch_len = 8;
    let mut glitch_str = String::with_capacity(glitch_len);
    for i in 0..glitch_len {
        let char_idx = (tick.wrapping_add((i * 13) as u64) as usize) % katakana.len();
        glitch_str.push(katakana[char_idx]);
    }

    let hero = Paragraph::new(Line::from(vec![
        Span::styled(
            format!("{} ", spinner),
            Style::default()
                .fg(Color::Rgb(200, 255, 200))
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            panel.headline.as_str(),
            Style::default()
                .fg(TEXT_PRIMARY)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(" [{}]", glitch_str),
            Style::default().fg(MATRIX_GREEN),
        ),
    ]))
    .alignment(Alignment::Left);
    f.render_widget(hero, chunks[0]);

    let detail = Paragraph::new(vec![
        Line::from(Span::styled(
            panel.detail.as_str(),
            Style::default().fg(TEXT_SECONDARY),
        )),
        Line::from(Span::styled(
            format!("raw status: {}", panel.raw_status),
            Style::default().fg(TEXT_DIM),
        )),
    ])
    .wrap(ratatui::widgets::Wrap { trim: true });
    f.render_widget(detail, chunks[1]);

    if let Some(pct) = panel.percent {
        let bar_width = (popup_area.width as usize)
            .saturating_sub(24)
            .max(12)
            .min(48);
        let filled = (pct * bar_width) / 100;
        let empty = bar_width.saturating_sub(filled);
        let bar = format!("{}{}", "█".repeat(filled), "░".repeat(empty));
        let ratio = panel
            .counts
            .map(|(current, total)| format!("{}/{}", current, total))
            .unwrap_or_else(|| "sync".to_string());
        let eta = panel
            .eta
            .clone()
            .unwrap_or_else(|| "estimating...".to_string());

        let bar_line = Paragraph::new(Line::from(vec![
            Span::styled(format!("[{}]", bar), Style::default().fg(SOFT_GREEN)),
            Span::styled(
                format!("  {:>3}%", pct),
                Style::default()
                    .fg(TEXT_PRIMARY)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(format!("  {}", ratio), Style::default().fg(TEXT_SECONDARY)),
            Span::styled(
                format!("  time left {}", eta),
                Style::default().fg(SOFT_GREEN),
            ),
        ]))
        .alignment(Alignment::Left);
        f.render_widget(bar_line, chunks[2]);
    }

    let status_line = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("phase", Style::default().fg(TEXT_DIM)),
            Span::styled("  first import sync", Style::default().fg(Color::White)),
        ]),
        Line::from(Span::styled(
            "What takes time: download -> decode -> dedupe/filter -> build the browseable index.",
            Style::default().fg(TEXT_SECONDARY),
        )),
    ])
    .wrap(ratatui::widgets::Wrap { trim: true });
    f.render_widget(status_line, chunks[3]);

    let eta_hint = Paragraph::new(Line::from(Span::styled(
        panel
            .eta
            .clone()
            .map(|eta| format!("Estimated remaining time: {}", eta))
            .unwrap_or_else(|| {
                "Estimated remaining time: calculating from live progress...".to_string()
            }),
        Style::default().fg(MATRIX_GREEN),
    )));
    f.render_widget(eta_hint, chunks[4]);

    let footer = Paragraph::new(Line::from(Span::styled(
        "esc to cancel",
        Style::default().fg(TEXT_DIM),
    )))
    .alignment(Alignment::Center);
    f.render_widget(footer, chunks[6]);
}

fn centered_rect(percent_x: u16, height: u16, r: Rect) -> Rect {
    let vertical_margin = r.height.saturating_sub(height) / 2;
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(vertical_margin),
            Constraint::Length(height),
            Constraint::Min(0),
        ])
        .split(r);

    let horizontal_margin = (100_u16.saturating_sub(percent_x)) / 2;
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(horizontal_margin),
            Constraint::Percentage(percent_x),
            Constraint::Min(0),
        ])
        .split(popup_layout[1])[1]
}

fn derive_loading_panel(app: &App) -> LoadingPanel {
    let raw_status = app
        .session
        .loading_message
        .clone()
        .unwrap_or_else(|| "Initializing system...".to_string());
    let progress_pct = app
        .session
        .loading_progress
        .as_ref()
        .and_then(|progress| {
            if progress.total > 0 {
                Some((progress.current * 100) / progress.total)
            } else {
                None
            }
        })
        .or_else(|| extract_percent(&raw_status));

    let counts = app
        .session
        .loading_progress
        .as_ref()
        .map(|progress| (progress.current, progress.total))
        .or_else(|| extract_ratio(&raw_status));

    let eta = app
        .session
        .loading_progress
        .as_ref()
        .and_then(|progress| progress.eta.as_ref().map(|d| format_duration(d.as_secs())))
        .or_else(|| extract_eta_seconds(&raw_status).map(format_duration));

    let (headline, detail) = if raw_status.contains("Downloading playlist") {
        (
            "Downloading playlist payload".to_string(),
            "Pulling the raw provider data down before it can be parsed into channels.".to_string(),
        )
    } else if raw_status.contains("Preparing memory mapping") || raw_status.contains("Received") {
        (
            "Staging playlist data in memory".to_string(),
            "The payload is fully downloaded. Matrix IPTV is preparing it for fast decoding."
                .to_string(),
        )
    } else if raw_status.contains("Deserializing JSON") {
        (
            "Decoding provider response".to_string(),
            "Turning the raw server payload into structured stream records.".to_string(),
        )
    } else if raw_status.contains("Preprocessing")
        || raw_status.contains("Optimizing")
        || raw_status.contains("Refining metadata")
    {
        (
            "Cleaning and organizing channels".to_string(),
            "Removing duplicates, applying playlist mode filters, and preparing metadata for browsing.".to_string(),
        )
    } else if raw_status.contains("Sorting")
        || raw_status.contains("Linking UI")
        || raw_status.contains("Finalized")
    {
        (
            "Building the interactive index".to_string(),
            "Final pass: sorting channels, wiring categories, and making the browser responsive."
                .to_string(),
        )
    } else if raw_status.contains("Loading all channels")
        || raw_status.contains("Fetching all channels")
    {
        (
            "Starting the first full channel sync".to_string(),
            "The first import is the slow one because every live channel has to be fetched and indexed.".to_string(),
        )
    } else if raw_status.contains("Loading categories") {
        (
            "Loading provider categories".to_string(),
            "Fetching the live category list so Matrix IPTV knows what it needs to scan next."
                .to_string(),
        )
    } else if raw_status.contains("Connecting to server")
        || raw_status.contains("Authenticating")
        || raw_status.contains("Processing Playlist")
    {
        (
            "Opening the provider session".to_string(),
            "Connecting, authenticating, and preparing the first import pipeline.".to_string(),
        )
    } else {
        (
            "Processing your playlist".to_string(),
            "Matrix IPTV is working through the provider data and will keep refining the ETA as more progress is available.".to_string(),
        )
    };

    let title = progress_pct
        .map(|pct| format!(" initial sync  {}% ", pct))
        .unwrap_or_else(|| " initial sync ".to_string());

    LoadingPanel {
        title,
        headline,
        detail,
        raw_status,
        percent: progress_pct,
        counts,
        eta,
    }
}

fn extract_percent(input: &str) -> Option<usize> {
    let percent_idx = input.find('%')?;
    let digits_rev: String = input[..percent_idx]
        .chars()
        .rev()
        .skip_while(|c| c.is_whitespace())
        .take_while(|c| c.is_ascii_digit())
        .collect();
    if digits_rev.is_empty() {
        None
    } else {
        digits_rev
            .chars()
            .rev()
            .collect::<String>()
            .parse::<usize>()
            .ok()
    }
}

fn extract_eta_seconds(input: &str) -> Option<u64> {
    let lower = input.to_lowercase();
    let start = lower.find("eta ")? + 4;
    let digits: String = lower[start..]
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect();
    if digits.is_empty() {
        None
    } else {
        digits.parse::<u64>().ok()
    }
}

fn extract_ratio(input: &str) -> Option<(usize, usize)> {
    let start = input.find('[')? + 1;
    let end = input[start..].find(']')? + start;
    let inside = &input[start..end];
    let (left, right) = inside.split_once('/')?;
    let current = left.trim().parse::<usize>().ok()?;
    let total = right.trim().parse::<usize>().ok()?;
    Some((current, total))
}

fn format_duration(secs: u64) -> String {
    if secs >= 60 {
        format!("{}m {}s", secs / 60, secs % 60)
    } else {
        format!("{}s", secs)
    }
}
