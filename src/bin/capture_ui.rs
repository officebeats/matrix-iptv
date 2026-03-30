use ratatui::{
    backend::TestBackend,
    Terminal,
};
use matrix_iptv_lib::app::{App, CurrentScreen, Pane};
use matrix_iptv_lib::api::{Category, Stream};
use matrix_iptv_lib::flex_id::FlexId;
use matrix_iptv_lib::ui::ui;
use std::sync::Arc;
use chrono::{Utc, Duration};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let backend = TestBackend::new(100, 30);
    let mut terminal = Terminal::new(backend)?;
    
    let mut app = App::new();
    app.current_screen = CurrentScreen::Streams;
    app.active_pane = Pane::Streams;
    
    // Create a mock NEWS category
    let mut news_cat = Category::default();
    news_cat.category_name = "US News & Politics".to_string();
    news_cat.category_id = "101".to_string();
    
    app.categories.push(Arc::new(news_cat));
    app.selected_category_index = 0;
    
    // Create 3 mock news streams
    let mut stream1 = Stream::default();
    stream1.name = "CNN Headline News".to_string();
    stream1.stream_id = FlexId::from_number(9001);
    
    let mut stream2 = Stream::default();
    stream2.name = "Fox News Channel".to_string();
    stream2.stream_id = FlexId::from_number(9002);
    
    let mut stream3 = Stream::default();
    stream3.name = "MSNBC Live".to_string();
    stream3.stream_id = FlexId::from_number(9003);
    
    app.streams.push(Arc::new(stream1));
    app.streams.push(Arc::new(stream2));
    app.streams.push(Arc::new(stream3));
    app.selected_stream_index = 0;
    app.stream_list_state.select(Some(0));
    
    // Mock EPG
    app.epg_cache.insert("9001".to_string(), "Anderson Cooper 360".to_string());
    
    app.state_loading = false;
    app.show_matrix_rain = false;
    
    terminal.draw(|f| {
        ui(f, &mut app);
    })?;

    let buffer = terminal.backend().buffer();
    for y in 0..30 {
        for x in 0..100 {
            let cell = buffer.cell((x, y));
            print!("{}", cell.expect("cell").symbol());
        }
        println!();
    }
    
    Ok(())
}
