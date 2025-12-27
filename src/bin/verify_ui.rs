use matrix_iptv_lib::parser::parse_stream;
use matrix_iptv_lib::ui::common::stylize_channel_name;
use matrix_iptv_lib::ui::colors::MATRIX_GREEN;
use ratatui::style::Color;

fn main() {
    let test_cases = vec![
        "EN - Before Sunrise (1995)",
        "EN | Before Sunrise (1995)",
        "EN/CAM - Before Sunrise (1995)",
        "The Notebook (2004)",
        "NBA TV (HD)",
    ];

    println!("=== UI Styling Verification ===\n");

    for name in test_cases {
        let parsed = parse_stream(name, None);
        println!("Original: \"{}\"", name);
        println!("  Cleaned Name: \"{}\"", parsed.display_name);

        let (spans, _) = stylize_channel_name(
            &parsed.display_name,
            false,
            false,
            parsed.quality,
            None,
            None,
            ratatui::style::Style::default().fg(MATRIX_GREEN),
        );

        print!("  Spans: ");
        for span in spans {
            let color = match span.style.fg {
                Some(c) => format_color(c),
                None => "Default".to_string(),
            };
            print!("[\"{}\" : {}] ", span.content, color);
        }
        println!("\n");
        
        // Assertions for "Before Sunrise (1995)"
        if name.contains("Before Sunrise") {
            assert!(!parsed.display_name.contains("EN"), "Should strip EN prefix");
            assert!(parsed.display_name.contains("Before Sunrise"), "Should contain title");
            assert!(parsed.display_name.contains("(1995)"), "Should preserve year");
        }
    }

    println!("✅ All programmatic styling tests passed verification.");
}

fn format_color(c: Color) -> String {
    if c == MATRIX_GREEN {
        "MATRIX_GREEN".to_string()
    } else if c == Color::White {
        "WHITE".to_string()
    } else if c == Color::Cyan {
        "CYAN".to_string()
    } else {
        format!("{:?}", c)
    }
}
