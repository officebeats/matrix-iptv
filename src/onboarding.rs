use ratatui::{
    backend::Backend,
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame, Terminal,
};
use crate::api::{IptvClient, XtreamClient};
use crate::config::{Account, AccountType, CategorySortOrder};
use std::collections::HashSet;
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::Duration;
use tachyonfx::{Effect, fx};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OnboardingStep {
    Welcome,
    HowToGet,
    Credentials,
    Validating,
    Success,
    Failed,
}

pub struct OnboardingState {
    pub step: OnboardingStep,
    pub input_name: String,
    pub input_url: String,
    pub input_username: String,
    pub input_password: String,
    pub active_field: usize, // 0 = Name, 1 = URL, 2 = User, 3 = Pass
    pub show_password: bool,
    pub error_msg: Option<String>,
    pub spinner_frame: usize,
    pub active_effect: Option<Effect>,
    pub should_quit: bool,
    pub success_account: Option<Account>,
    pub validation_rx: Option<Receiver<Result<(String, String, String, String), String>>>,
}

impl Default for OnboardingState {
    fn default() -> Self {
        Self {
            step: OnboardingStep::Welcome,
            input_name: "My Playlist".to_string(),
            input_url: "".to_string(),
            input_username: "".to_string(),
            input_password: "".to_string(),
            active_field: 1, // Start at URL
            show_password: false,
            error_msg: None,
            spinner_frame: 0,
            active_effect: Some(fx::dissolve(600u32)),
            should_quit: false,
            success_account: None,
            validation_rx: None,
        }
    }
}

pub fn run_onboarding<B: Backend>(terminal: &mut Terminal<B>) -> anyhow::Result<Option<Account>> 
where 
    <B as Backend>::Error: std::fmt::Display + std::fmt::Debug + Send + Sync + 'static
{
    let mut state = OnboardingState::default();
    let tick_rate = Duration::from_millis(16); // 60fps for smooth animations

    loop {
        terminal.draw(|f| ui(f, &mut state)).map_err(|e| anyhow::anyhow!(e))?;

        if state.should_quit {
            return Ok(None);
        }
        
        if let Some(_account) = state.success_account.clone() {
            // Need a slight pause for completion state to be seen
            if state.step == OnboardingStep::Success {
                 // Wait for user to press enter in handle_key
            }
        }

        if crossterm::event::poll(tick_rate)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    handle_key(&mut state, key.code);
                    if state.step == OnboardingStep::Success && key.code == KeyCode::Enter {
                        return Ok(state.success_account.take());
                    }
                }
            }
        }

        // Check for validation result if in Validating step
        if state.step == OnboardingStep::Validating {
            state.spinner_frame = (state.spinner_frame + 1) % 10;

            if let Some(rx) = &state.validation_rx {
                if let Ok(result) = rx.try_recv() {
                    match result {
                        Ok((url, username, password, name)) => {
                            state.success_account = Some(Account {
                                name: if name.is_empty() { "My Playlist".to_string() } else { name },
                                base_url: url,
                                username,
                                password,
                                account_type: AccountType::Xtream,
                                epg_url: None,
                                last_refreshed: None,
                                total_channels: None,
                                total_movies: None,
                                total_series: None,
                                server_timezone: None,
                                hidden_categories: HashSet::new(),
                                category_sort_order: CategorySortOrder::Default,
                            });
                            transition_to(&mut state, OnboardingStep::Success);
                        }
                        Err(e) => {
                            state.error_msg = Some(e);
                            transition_to(&mut state, OnboardingStep::Failed);
                        }
                    }
                }
            }
        }
    }
}

fn handle_key(state: &mut OnboardingState, key: KeyCode) {
    match state.step {
        OnboardingStep::Welcome => {
            match key {
                KeyCode::Enter => transition_to(state, OnboardingStep::HowToGet),
                KeyCode::Esc => state.should_quit = true,
                _ => {}
            }
        }
        OnboardingStep::HowToGet => {
            match key {
                KeyCode::Esc | KeyCode::Left => transition_to(state, OnboardingStep::Welcome),
                KeyCode::Enter | KeyCode::Right => transition_to(state, OnboardingStep::Credentials),
                KeyCode::Char('o') | KeyCode::Char('O') => {
                    let _ = webbrowser::open("https://g2g.to/iptv");
                },
                _ => {}
            }
        }
        OnboardingStep::Credentials => {
            match key {
                KeyCode::Esc => transition_to(state, OnboardingStep::HowToGet),
                KeyCode::Tab | KeyCode::Down => {
                    state.active_field = (state.active_field + 1) % 4;
                }
                KeyCode::BackTab | KeyCode::Up => {
                    if state.active_field == 0 {
                        state.active_field = 3;
                    } else {
                        state.active_field -= 1;
                    }
                }
                KeyCode::Char('t') | KeyCode::Char('T') if state.active_field != 3 => {
                    // If not in password field, 't' can be typed OR used as toggle?
                    // Let's just allow typing for now, and have a dedicated key if needed.
                    // Actually, let's just use Ctrl-T for toggle or something safer.
                    // Or only toggle if not focused on a text field? but they are always focused.
                    // Standard pattern: keep it simple.
                    append_char(state, 't');
                }
                KeyCode::Char('T') if state.active_field != 3 => append_char(state, 'T'),
                KeyCode::Char('t') | KeyCode::Char('T') if state.active_field == 3 => {
                    state.show_password = !state.show_password;
                }
                KeyCode::Enter => {
                    if state.input_url.is_empty() || state.input_username.is_empty() || state.input_password.is_empty() {
                        state.error_msg = Some("Please fill in all fields".to_string());
                    } else {
                        state.error_msg = None;
                        state.spinner_frame = 0;
                        state.step = OnboardingStep::Validating;
                        
                        // Trigger real validation task
                        let url = state.input_url.clone();
                        let username = state.input_username.clone();
                        let password = state.input_password.clone();
                        let name = state.input_name.clone();
                    
                        let (tx, rx) = mpsc::channel();
                        state.validation_rx = Some(rx);
                        
                        thread::spawn(move || {
                            let rt = tokio::runtime::Builder::new_current_thread()
                                .enable_all()
                                .build()
                                .unwrap();
                            rt.block_on(async {
                                let client = IptvClient::Xtream(XtreamClient::new(url.clone(), username.clone(), password.clone()));
                                match client.authenticate().await {
                                    Ok((true, _, _, _)) => {
                                        let _ = tx.send(Ok((url, username, password, name)));
                                    }
                                    Ok((false, _, _, _)) => {
                                        let _ = tx.send(Err("Invalid credentials".to_string()));
                                    }
                                    Err(e) => {
                                        let _ = tx.send(Err(e.to_string()));
                                    }
                                }
                            });
                        });
                    }
                }
                KeyCode::Backspace => {
                    let field = match state.active_field {
                        0 => &mut state.input_name,
                        1 => &mut state.input_url,
                        2 => &mut state.input_username,
                        _ => &mut state.input_password,
                    };
                    field.pop();
                }
                KeyCode::Char(c) => {
                    append_char(state, c);
                }
                _ => {}
            }
        }
        OnboardingStep::Validating => {}
        OnboardingStep::Failed => {
            match key {
                KeyCode::Esc | KeyCode::Enter => transition_to(state, OnboardingStep::Credentials),
                _ => {}
            }
        }
        OnboardingStep::Success => {}
    }
}

fn append_char(state: &mut OnboardingState, c: char) {
    let field = match state.active_field {
        0 => &mut state.input_name,
        1 => &mut state.input_url,
        2 => &mut state.input_username,
        _ => &mut state.input_password,
    };
    field.push(c);
}

fn transition_to(state: &mut OnboardingState, next_step: OnboardingStep) {
    state.step = next_step;
    state.active_effect = Some(fx::dissolve(300u32));
}

fn ui(f: &mut Frame, state: &mut OnboardingState) {
    let area = f.area();
    f.render_widget(Clear, f.area());
    
    // Explicit background fill to prevent terminal "gray" artifacting
    f.render_widget(
        ratatui::widgets::Block::default()
            .bg(ratatui::style::Color::Rgb(0, 0, 0)),
        f.area()
    );

    let content_area = centered_rect(70, 70, area);
    
    match state.step {
        OnboardingStep::Welcome => render_welcome(f, state, content_area),
        OnboardingStep::HowToGet => render_how_to_get(f, state, content_area),
        OnboardingStep::Credentials => render_credentials(f, state, content_area),
        OnboardingStep::Validating => render_validating(f, state, content_area),
        OnboardingStep::Failed => render_failed(f, state, content_area),
        OnboardingStep::Success => render_success(f, state, content_area),
    }

    render_footer(f, state, area);
}

fn render_welcome(f: &mut Frame, _state: &OnboardingState, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(crate::ui::colors::SOFT_GREEN))
        .border_type(ratatui::widgets::BorderType::Rounded)
        .title(Span::styled(" MATRIX IPTV ONBOARDING ", Style::default().fg(crate::ui::colors::MATRIX_GREEN).add_modifier(Modifier::BOLD)))
        .title_alignment(Alignment::Center);

    let inner = block.inner(area);
    f.render_widget(block, area);

    let ascii_logo = r#"
        ███╗   ███╗ █████╗ ████████╗██████╗ ██╗██╗  ██╗
        ████╗ ████║██╔══██╗╚══██╔══╝██╔══██╗██║╚██╗██╔╝
        ██╔████╔██║███████║   ██║   ██████╔╝██║ ╚███╔╝ 
        ██║╚██╔╝██║██╔══██║   ██║   ██╔══██╗██║ ██╔██╗ 
        ██║ ╚═╝ ██║██║  ██║   ██║   ██║  ██║██║██╔╝ ██╗
        ╚═╝     ╚═╝╚═╝  ╚═╝   ╚═╝   ╚═╝  ╚═╝╚═╝╚═╝  ╚═╝
"#;

    let text = vec![
        Line::from(""),
        Line::from(Span::styled(ascii_logo, Style::default().fg(crate::ui::colors::MATRIX_GREEN))),
        Line::from(""),
        Line::from("Welcome to Matrix IPTV!").alignment(Alignment::Center),
        Line::from("The ultimate TUI for your streaming needs.").alignment(Alignment::Center),
        Line::from(""),
        Line::from("You don't have any playlists configured yet.").alignment(Alignment::Center),
        Line::from(""),
        Line::from(Span::styled("Press [Enter] to begin setup", Style::default().fg(crate::ui::colors::TEXT_SECONDARY))),
    ];
    
    let p = Paragraph::new(text).alignment(Alignment::Center);
    f.render_widget(p, inner);
}

fn render_how_to_get(f: &mut Frame, _state: &OnboardingState, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(crate::ui::colors::SOFT_GREEN))
        .border_type(ratatui::widgets::BorderType::Rounded)
        .title(Span::styled(" STEP 1: GET A PLAYLIST ", Style::default().fg(crate::ui::colors::MATRIX_GREEN).add_modifier(Modifier::BOLD)))
        .title_alignment(Alignment::Center);

    let inner = block.inner(area);
    f.render_widget(block, area);

    let text = vec![
        Line::from(""),
        Line::from(Span::styled("Matrix IPTV is a player. You need a provider.", Style::default().add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from("If you don't have one, search online for IPTV providers."),
        Line::from("Popular sources: G2G, Z2U, eBay."),
        Line::from("Highly rated: Strong8k, Mega IPTV."),
        Line::from(""),
        Line::from("Look for \"Xtream Codes\" format credentials:"),
        Line::from("URL, Username, and Password."),
        Line::from(""),
        Line::from(Span::styled("Press [O] to open search in browser", Style::default().fg(crate::ui::colors::TEXT_SECONDARY))),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press [Enter] to continue", Style::default().fg(crate::ui::colors::MATRIX_GREEN).add_modifier(Modifier::BOLD)),
            Span::styled(" | ", Style::default().fg(Color::DarkGray)),
            Span::styled("[Esc] Go Back", Style::default().fg(Color::Gray)),
        ]),
    ];
    
    let p = Paragraph::new(text).alignment(Alignment::Center).wrap(Wrap { trim: false });
    f.render_widget(p, inner);
}

fn render_credentials(f: &mut Frame, state: &OnboardingState, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(crate::ui::colors::SOFT_GREEN))
        .border_type(ratatui::widgets::BorderType::Rounded)
        .title(Span::styled(" STEP 2: ENTER CREDENTIALS ", Style::default().fg(crate::ui::colors::MATRIX_GREEN).add_modifier(Modifier::BOLD)))
        .title_alignment(Alignment::Center);

    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Name
            Constraint::Length(3), // URL
            Constraint::Length(3), // User
            Constraint::Length(3), // Pass
            Constraint::Min(1),    // Error/Hints
        ])
        .split(inner);

    let active_style = Style::default().fg(crate::ui::colors::MATRIX_GREEN).add_modifier(Modifier::BOLD);
    let inactive_style = Style::default().fg(Color::DarkGray);

    // Name
    f.render_widget(
        Paragraph::new(state.input_name.as_str())
            .block(Block::default().borders(Borders::ALL).title(" Playlist Name (Optional) ").border_style(if state.active_field==0 {active_style} else {inactive_style})),
        chunks[0]
    );

    // URL
    f.render_widget(
        Paragraph::new(state.input_url.as_str())
            .block(Block::default().borders(Borders::ALL).title(" Server URL (http://...) ").border_style(if state.active_field==1 {active_style} else {inactive_style})),
        chunks[1]
    );

    // User
    f.render_widget(
        Paragraph::new(state.input_username.as_str())
            .block(Block::default().borders(Borders::ALL).title(" Username ").border_style(if state.active_field==2 {active_style} else {inactive_style})),
        chunks[2]
    );

    // Pass
    let pass_display = if state.show_password { state.input_password.clone() } else { "*".repeat(state.input_password.len()) };
    f.render_widget(
        Paragraph::new(pass_display)
            .block(Block::default().borders(Borders::ALL).title(" Password ").border_style(if state.active_field==3 {active_style} else {inactive_style})),
        chunks[3]
    );

    // Hints & Error
    let mut hints = vec![
        Line::from(vec![
            Span::styled("[Tab]", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" Move   "),
            Span::styled("[T]", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" Toggle Pass (at field)   "),
            Span::styled("[Enter]", Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(" Connect", Style::default().fg(crate::ui::colors::MATRIX_GREEN)),
        ])
    ];

    if let Some(err) = &state.error_msg {
        hints.push(Line::from(""));
        hints.push(Line::from(Span::styled(format!(" ERROR: {} ", err), Style::default().bg(Color::Red).fg(Color::White).add_modifier(Modifier::BOLD))));
    }

    f.render_widget(Paragraph::new(hints).alignment(Alignment::Center), chunks[4]);
}

fn render_validating(f: &mut Frame, state: &OnboardingState, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(crate::ui::colors::MATRIX_GREEN))
        .border_type(ratatui::widgets::BorderType::Rounded)
        .title(" CONNECTING ");
    
    let inner = block.inner(area);
    f.render_widget(block, area);

    let frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    let spinner = frames[state.spinner_frame % frames.len()];

    let text = vec![
        Line::from(""),
        Line::from(Span::styled(format!("{} Validating Server Connection...", spinner), Style::default().fg(crate::ui::colors::MATRIX_GREEN))),
        Line::from(""),
        Line::from("Checking credentials and downloading basics..."),
    ];

    f.render_widget(Paragraph::new(text).alignment(Alignment::Center), inner);
}

fn render_failed(f: &mut Frame, state: &OnboardingState, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red))
        .border_type(ratatui::widgets::BorderType::Rounded)
        .title(" CONNECTION FAILED ");
    
    let inner = block.inner(area);
    f.render_widget(block, area);

    let text = vec![
        Line::from(""),
        Line::from(Span::styled("X Failed to authenticate with the server.", Style::default().fg(Color::Red))),
        Line::from(""),
        Line::from(state.error_msg.as_deref().unwrap_or("Unknown error")),
        Line::from(""),
        Line::from(Span::styled("Press [Enter] to try again", Style::default().add_modifier(Modifier::BOLD))),
    ];

    f.render_widget(Paragraph::new(text).alignment(Alignment::Center), inner);
}

fn render_success(f: &mut Frame, _state: &OnboardingState, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(crate::ui::colors::MATRIX_GREEN))
        .border_type(ratatui::widgets::BorderType::Rounded)
        .title(" SUCCESS ");
    
    let inner = block.inner(area);
    f.render_widget(block, area);

    let text = vec![
        Line::from(""),
        Line::from(Span::styled("✓ Connected successfully!", Style::default().fg(crate::ui::colors::MATRIX_GREEN).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from("Your first playlist has been configured."),
        Line::from("Welcome to the Matrix."),
        Line::from(""),
        Line::from(Span::styled("Press [Enter] to start", Style::default().fg(crate::ui::colors::MATRIX_GREEN).add_modifier(Modifier::BOLD))),
    ];

    f.render_widget(Paragraph::new(text).alignment(Alignment::Center), inner);
}

fn render_footer(f: &mut Frame, state: &OnboardingState, area: Rect) {
    let current = match state.step {
        OnboardingStep::Welcome => 1,
        OnboardingStep::HowToGet => 2,
        OnboardingStep::Credentials | OnboardingStep::Validating | OnboardingStep::Failed => 3,
        OnboardingStep::Success => 4,
    };
    let dots = (1..=4).map(|i| if i == current { "●" } else { "○" }).collect::<Vec<_>>().join(" ");
    
    let footer_text = format!(" Step {} of 4  {} ", current, dots);
    let area = Rect {
        x: area.x,
        y: area.y + area.height - 1,
        width: area.width,
        height: 1,
    };
    f.render_widget(Paragraph::new(footer_text).fg(Color::DarkGray), area);
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
