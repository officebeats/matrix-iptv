use ratatui::layout::{Constraint, Direction, Layout, Rect};
use crate::api::{Category, Stream, SeriesEpisode};
use std::sync::Arc;

pub fn calculate_max_category_width(categories: &[Arc<Category>], total_width: u16) -> u16 {
    if categories.is_empty() {
        return 25; // Minimum default width
    }
    
    let max_content = categories
        .iter()
        .map(|c| {
            (c.category_name.len() as u16) + 5
        })
        .max()
        .unwrap_or(25);

    let dynamic_max = (total_width * 40 / 100).max(45);
    
    max_content
        .max(25) 
        .min(dynamic_max) 
}

pub fn calculate_two_column_split(categories: &[Arc<Category>], total_width: u16) -> (u16, u16) {
    let cat_width = calculate_max_category_width(categories, total_width);
    let min_stream_width = 60; 
    
    if cat_width + min_stream_width > total_width {
        (total_width * 30 / 100, total_width * 70 / 100)
    } else {
        (cat_width, total_width - cat_width)
    }
}

pub fn calculate_three_column_split(
    categories: &[Arc<Category>],
    series: &[Arc<Stream>],
    episodes: &[SeriesEpisode],
    total_width: u16,
) -> (u16, u16, u16) {
    let cat_width = calculate_max_category_width(categories, total_width);
    
    let series_max_content = if series.is_empty() {
        35
    } else {
        series
            .iter()
            .map(|s| {
                (s.name.len() as u16) + 13
            })
            .max()
            .unwrap_or(35)
    };
    
    let series_dynamic_max = (total_width * 35 / 100).max(45);
    let series_width = series_max_content.max(35).min(series_dynamic_max);
    
    let episode_max_content = if episodes.is_empty() {
        45
    } else {
        episodes
            .iter()
            .map(|ep| {
                let title = ep.title.as_deref().unwrap_or("Untitled");
                (title.len() as u16) + 12
            })
            .max()
            .unwrap_or(45)
    };

    let min_episode_width = 50;
    let episode_width = episode_max_content.max(min_episode_width);
    
    let total_needed = cat_width + series_width + episode_width;
    
    if total_needed > total_width {
        (total_width * 25 / 100, total_width * 35 / 100, total_width * 40 / 100)
    } else {
        let remaining = total_width - cat_width - series_width;
        (cat_width, series_width, remaining.max(episode_width))
    }
}

pub fn calculate_vod_three_column_split(
    categories: &[Arc<Category>],
    streams: &[Arc<Stream>],
    total_width: u16,
) -> (u16, u16, u16) {
    let cat_width = calculate_max_category_width(categories, total_width);
    
    let stream_max_content = if streams.is_empty() {
        35
    } else {
        streams
            .iter()
            .map(|s| {
                (s.name.len() as u16) + 13
            })
            .max()
            .unwrap_or(35)
    };
    
    let stream_dynamic_max = (total_width * 35 / 100).max(45);
    let stream_width = stream_max_content.max(35).min(stream_dynamic_max);
    
    let details_width = 50; 
    
    let total_needed = cat_width + stream_width + details_width;
    
    if total_needed > total_width {
        (total_width * 25 / 100, total_width * 35 / 100, total_width * 40 / 100)
    } else {
        let remaining = total_width - cat_width - stream_width;
        (cat_width, stream_width, remaining)
    }
}

pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
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
pub fn get_rating_color(rating: &str) -> ratatui::style::Color {
    if let Ok(r) = rating.parse::<f32>() {
        if r >= 8.0 {
            ratatui::style::Color::Green
        } else if r >= 6.0 {
            ratatui::style::Color::White
        } else if r >= 4.0 {
            ratatui::style::Color::LightYellow
        } else {
            ratatui::style::Color::Red
        }
    } else {
        ratatui::style::Color::White
    }
}
