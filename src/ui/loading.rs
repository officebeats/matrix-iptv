use crate::app::App;
use crate::ui::colors::{MATRIX_GREEN, SOFT_GREEN, TEXT_DIM, TEXT_PRIMARY, TEXT_SECONDARY};
use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
};

const BRAILLE_SPINNER: &[char] = &['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];

pub fn get_loading_status_line(app: &App) -> Option<Line<'static>> {
    if !app.session.state_loading || app.session.loading_message.is_none() {
        return None;
    }

    let tick = app.session.loading_tick;
    let spinner = BRAILLE_SPINNER[(tick as usize) % BRAILLE_SPINNER.len()];

    let panel = derive_loading_info(app);

    let mut spans: Vec<Span> = Vec::new();

    spans.push(Span::styled(
        format!("{} ", spinner),
        Style::default()
            .fg(MATRIX_GREEN)
            .add_modifier(Modifier::BOLD),
    ));

    spans.push(Span::styled(
        format!("{} ", panel.verb),
        Style::default()
            .fg(TEXT_PRIMARY)
            .add_modifier(Modifier::BOLD),
    ));

    if let Some(pct) = panel.percent {
        let bar_width = 12usize.min(20).max(8);
        let filled = (pct * bar_width) / 100;
        let empty = bar_width.saturating_sub(filled);
        let bar = format!(
            "[{}{}]",
            "█".repeat(filled.min(bar_width)),
            "░".repeat(empty.min(bar_width))
        );
        spans.push(Span::styled(
            format!("{} {:>3}% ", bar, pct),
            Style::default().fg(SOFT_GREEN),
        ));
    }

    if let Some((current, total)) = panel.counts {
        spans.push(Span::styled(
            format!("{}/{} ", current, total),
            Style::default().fg(TEXT_SECONDARY),
        ));
    }

    if let Some(eta) = &panel.eta {
        spans.push(Span::styled(
            format!("ETA {} ", eta),
            Style::default().fg(SOFT_GREEN),
        ));
    }

    spans.push(Span::styled("│  esc cancel", Style::default().fg(TEXT_DIM)));

    Some(Line::from(spans))
}

struct LoadingInfo {
    verb: String,
    percent: Option<usize>,
    counts: Option<(usize, usize)>,
    eta: Option<String>,
}

fn derive_loading_info(app: &App) -> LoadingInfo {
    let raw_status = app
        .session
        .loading_message
        .clone()
        .unwrap_or_else(|| "Processing...".to_string());

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

    let verb = if raw_status.contains("Downloading playlist") {
        "Downloading playlist".to_string()
    } else if raw_status.contains("Preparing memory mapping") || raw_status.contains("Received") {
        "Staging data in memory".to_string()
    } else if raw_status.contains("Deserializing JSON") {
        "Decoding provider response".to_string()
    } else if raw_status.contains("Preprocessing")
        || raw_status.contains("Optimizing")
        || raw_status.contains("Refining metadata")
    {
        "Cleaning and organizing".to_string()
    } else if raw_status.contains("Sorting")
        || raw_status.contains("Linking UI")
        || raw_status.contains("Finalized")
    {
        "Building index".to_string()
    } else if raw_status.contains("Loading all channels")
        || raw_status.contains("Fetching all channels")
    {
        "Syncing channels".to_string()
    } else if raw_status.contains("Loading categories") {
        "Loading categories".to_string()
    } else if raw_status.contains("Connecting to server")
        || raw_status.contains("Authenticating")
        || raw_status.contains("Processing Playlist")
    {
        "Connecting to server".to_string()
    } else {
        "Processing".to_string()
    };

    LoadingInfo {
        verb,
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
