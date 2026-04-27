use crate::api::{IptvClient, M3uClient, XtreamClient};
use crate::config::{Account, AccountType, CategorySortOrder};
use ratatui::{
    backend::Backend,
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame, Terminal,
};
use std::collections::HashSet;
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::Duration;
use tachyonfx::{fx, Effect};

type ValidationResult = Result<(String, String, String, String), String>;
type ValidationReceiver = Receiver<ValidationResult>;

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
    pub validation_rx: Option<ValidationReceiver>,
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
    <B as Backend>::Error: std::fmt::Display + std::fmt::Debug + Send + Sync + 'static,
{
    let mut state = OnboardingState::default();
    let tick_rate = Duration::from_millis(16); // 60fps for smooth animations

    loop {
        terminal
            .draw(|f| ui(f, &mut state))
            .map_err(|e| anyhow::anyhow!(e))?;

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
                                name: if name.is_empty() {
                                    "My Playlist".to_string()
                                } else {
                                    name
                                },
                                base_url: url.clone(),
                                username: username.clone(),
                                password: password.clone(),
                                account_type: if crate::app::App::is_m3u_url(
                                    &url, &username, &password,
                                ) {
                                    AccountType::M3uUrl
                                } else {
                                    AccountType::Xtream
                                },
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
        OnboardingStep::Welcome => match key {
            KeyCode::Enter => transition_to(state, OnboardingStep::HowToGet),
            KeyCode::Esc => state.should_quit = true,
            _ => {}
        },
        OnboardingStep::HowToGet => match key {
            KeyCode::Esc | KeyCode::Left => transition_to(state, OnboardingStep::Welcome),
            KeyCode::Enter | KeyCode::Right => transition_to(state, OnboardingStep::Credentials),
            KeyCode::Char('o') | KeyCode::Char('O') => {
                let _ = webbrowser::open("https://g2g.to/iptv");
            }
            _ => {}
        },
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
                    if state.input_url.is_empty() {
                        state.error_msg = Some("Please fill in the URL field".to_string());
                    } else if state.input_username.is_empty()
                        && state.input_password.is_empty()
                        && !crate::app::App::is_m3u_url(
                            &state.input_url,
                            &state.input_username,
                            &state.input_password,
                        )
                    {
                        state.error_msg = Some("Please fill in all fields (or leave username/password empty for M3U URL)".to_string());
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

                        let is_m3u = crate::app::App::is_m3u_url(&url, &username, &password);

                        thread::spawn(move || {
                            let rt = tokio::runtime::Builder::new_current_thread()
                                .enable_all()
                                .build()
                                .unwrap();
                            rt.block_on(async {
                                if is_m3u {
                                    let client = IptvClient::M3u(M3uClient::new(url.clone()));
                                    match client.authenticate().await {
                                        Ok((true, _, _, _)) => {
                                            let _ = tx.send(Ok((url, username, password, name)));
                                        }
                                        Ok((false, _, _, _)) => {
                                            let _ = tx
                                                .send(Err("Invalid M3U playlist URL or format"
                                                    .to_string()));
                                        }
                                        Err(e) => {
                                            let _ = tx.send(Err(e.to_string()));
                                        }
                                    }
                                } else {
                                    let client = IptvClient::Xtream(XtreamClient::new(
                                        url.clone(),
                                        username.clone(),
                                        password.clone(),
                                    ));
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
        OnboardingStep::Failed => match key {
            KeyCode::Esc | KeyCode::Enter => transition_to(state, OnboardingStep::Credentials),
            _ => {}
        },
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
        ratatui::widgets::Block::default().bg(ratatui::style::Color::Rgb(0, 0, 0)),
        f.area(),
    );

    let margin_v = std::cmp::max(1, area.height.saturating_mul(12) / 100);
    let margin_h = std::cmp::max(2, area.width.saturating_mul(16) / 100);
    crate::matrix_rain::render_matrix_edge_border(f, area, margin_v, margin_h);

    let framed_area = Rect {
        x: area.x.saturating_add(margin_h),
        y: area.y.saturating_add(margin_v),
        width: area.width.saturating_sub(margin_h * 2),
        height: area.height.saturating_sub(margin_v * 2),
    };
    let content_area = centered_rect(78, 82, framed_area);

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

fn render_shell<'a>(
    f: &mut Frame,
    area: Rect,
    step_label: &'a str,
    title: &'a str,
    subtitle: &'a str,
) -> Rect {
    let inner = crate::ui::common::render_composite_block(
        f,
        area,
        Some(" // FIRST-TIME PLAYLIST SETUP // "),
    );
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(4), Constraint::Min(0)])
        .split(inner);

    let header = Paragraph::new(vec![
        Line::from(vec![
            Span::styled(
                step_label,
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("  //  ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                title,
                Style::default()
                    .fg(crate::ui::colors::MATRIX_GREEN)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            subtitle,
            Style::default().fg(crate::ui::colors::TEXT_SECONDARY),
        )),
    ]);
    f.render_widget(header, chunks[0]);
    chunks[1]
}

fn render_welcome(f: &mut Frame, _state: &OnboardingState, area: Rect) {
    let inner = render_shell(
        f,
        area,
        "STEP 1/4",
        "bootstrap your first provider connection",
        "The FTUE now mirrors the animated home-screen framing so setup feels like part of the product, not a dead-end form.",
    );

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(10),
            Constraint::Length(7),
            Constraint::Length(4),
            Constraint::Min(0),
        ])
        .split(inner);

    let ascii_logo = r#"
        ███╗   ███╗ █████╗ ████████╗██████╗ ██╗██╗  ██╗
        ████╗ ████║██╔══██╗╚══██╔══╝██╔══██╗██║╚██╗██╔╝
        ██╔████╔██║███████║   ██║   ██████╔╝██║ ╚███╔╝ 
        ██║╚██╔╝██║██╔══██║   ██║   ██╔══██╗██║ ██╔██╗ 
        ██║ ╚═╝ ██║██║  ██║   ██║   ██║  ██║██║██╔╝ ██╗
        ╚═╝     ╚═╝╚═╝  ╚═╝   ╚═╝   ╚═╝  ╚═╝╚═╝╚═╝  ╚═╝
"#;

    let hero = Paragraph::new(vec![
        Line::from(Span::styled(ascii_logo, Style::default().fg(crate::ui::colors::MATRIX_GREEN))),
        Line::from(""),
        Line::from(Span::styled("Welcome to Matrix IPTV.", Style::default().fg(Color::White).add_modifier(Modifier::BOLD))).alignment(Alignment::Center),
        Line::from("You do not have any playlists configured yet, so this setup will walk you from provider discovery to a working first import.").alignment(Alignment::Center),
    ])
    .alignment(Alignment::Center);
    f.render_widget(hero, chunks[0]);

    let cards = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(34),
            Constraint::Percentage(33),
        ])
        .split(chunks[1]);

    let card_text = [
        (
            "01 // source",
            "Find an IPTV provider that gives you Xtream Codes credentials.",
        ),
        (
            "02 // connect",
            "Enter server URL, username, and password with inline guidance.",
        ),
        (
            "03 // import",
            "Let Matrix IPTV validate the login and prepare your first library.",
        ),
    ];

    for (idx, (title, body)) in card_text.iter().enumerate() {
        let card_inner = crate::ui::common::render_matrix_box(
            f,
            cards[idx],
            title,
            crate::ui::colors::SOFT_GREEN,
        );
        f.render_widget(
            Paragraph::new(vec![
                Line::from(""),
                Line::from(Span::styled(
                    *body,
                    Style::default().fg(crate::ui::colors::TEXT_PRIMARY),
                )),
            ])
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true }),
            card_inner,
        );
    }

    let action = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("[Enter]", Style::default().fg(crate::ui::colors::MATRIX_GREEN).add_modifier(Modifier::BOLD)),
            Span::styled(" begin setup   ", Style::default().fg(Color::White)),
            Span::styled("[Esc]", Style::default().fg(crate::ui::colors::TEXT_SECONDARY).add_modifier(Modifier::BOLD)),
            Span::styled(" exit", Style::default().fg(crate::ui::colors::TEXT_SECONDARY)),
        ]),
        Line::from(""),
        Line::from(Span::styled("Tip: the first full import is the longest one. After that, cached startup is much faster.", Style::default().fg(crate::ui::colors::TEXT_SECONDARY))),
    ])
    .alignment(Alignment::Center);
    f.render_widget(action, chunks[2]);
}

fn render_how_to_get(f: &mut Frame, _state: &OnboardingState, area: Rect) {
    let inner = render_shell(
        f,
        area,
        "STEP 2/4",
        "get the right credentials before you continue",
        "Matrix IPTV is only the player. You bring the provider and credentials, and the app handles playback, browsing, and caching.",
    );

    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(48), Constraint::Percentage(52)])
        .split(inner);

    let left = crate::ui::common::render_matrix_box(
        f,
        columns[0],
        "what to buy",
        crate::ui::colors::SOFT_GREEN,
    );
    let left_text = Paragraph::new(vec![
        Line::from(Span::styled(
            "Required format",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("• Server URL"),
        Line::from("• Username"),
        Line::from("• Password"),
        Line::from(""),
        Line::from(Span::styled(
            "Ask specifically for Xtream Codes credentials.",
            Style::default().fg(crate::ui::colors::TEXT_SECONDARY),
        )),
        Line::from("M3U-only access can work elsewhere, but this FTUE expects Xtream credentials."),
    ])
    .wrap(Wrap { trim: true });
    f.render_widget(left_text, left);

    let right = crate::ui::common::render_matrix_box(
        f,
        columns[1],
        "where to look",
        crate::ui::colors::SOFT_GREEN,
    );
    let right_text = Paragraph::new(vec![
        Line::from(Span::styled(
            "Common marketplaces",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("• G2G"),
        Line::from("• Z2U"),
        Line::from("• eBay"),
        Line::from(""),
        Line::from(Span::styled(
            "Frequently mentioned providers",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("• Strong8k"),
        Line::from("• Mega IPTV"),
        Line::from(""),
        Line::from(Span::styled(
            "[O] opens a browser search for you.",
            Style::default().fg(crate::ui::colors::TEXT_SECONDARY),
        )),
        Line::from(Span::styled(
            "[Enter] continues once you already have the credentials.",
            Style::default().fg(crate::ui::colors::TEXT_SECONDARY),
        )),
    ])
    .wrap(Wrap { trim: true });
    f.render_widget(right_text, right);
}

fn render_credentials(f: &mut Frame, state: &OnboardingState, area: Rect) {
    let inner = render_shell(
        f,
        area,
        "STEP 3/4",
        "enter credentials and validate the provider",
        "Use the exact values from your provider. The connection test only proceeds once URL, username, and password are present.",
    );

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(33), Constraint::Percentage(67)])
        .split(inner);

    let tips = crate::ui::common::render_matrix_box(
        f,
        chunks[0],
        "operator notes",
        crate::ui::colors::SOFT_GREEN,
    );
    let tips_text = Paragraph::new(vec![
        Line::from(Span::styled(
            "Expected input",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("• Playlist name is optional."),
        Line::from("• Server URL should look like http://provider.example:8080"),
        Line::from("• Username and password must match exactly."),
        Line::from(""),
        Line::from(Span::styled(
            "Keyboard",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("• [Tab] or [Down] moves forward"),
        Line::from("• [Shift+Tab] or [Up] moves backward"),
        Line::from("• [T] toggles password visibility while Password is focused"),
        Line::from("• [Enter] starts validation"),
    ])
    .wrap(Wrap { trim: true });
    f.render_widget(tips_text, tips);

    let form_inner = crate::ui::common::render_matrix_box(
        f,
        chunks[1],
        "connection form",
        crate::ui::colors::SOFT_GREEN,
    );
    let form_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(1),
        ])
        .split(form_inner);

    let active_style = Style::default()
        .fg(crate::ui::colors::MATRIX_GREEN)
        .add_modifier(Modifier::BOLD);
    let inactive_style = Style::default().fg(Color::DarkGray);

    f.render_widget(
        Paragraph::new(state.input_name.as_str()).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Playlist Name (Optional) ")
                .border_style(if state.active_field == 0 {
                    active_style
                } else {
                    inactive_style
                }),
        ),
        form_chunks[0],
    );

    f.render_widget(
        Paragraph::new(state.input_url.as_str()).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Server URL (http://...) ")
                .border_style(if state.active_field == 1 {
                    active_style
                } else {
                    inactive_style
                }),
        ),
        form_chunks[1],
    );

    f.render_widget(
        Paragraph::new(state.input_username.as_str()).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Username ")
                .border_style(if state.active_field == 2 {
                    active_style
                } else {
                    inactive_style
                }),
        ),
        form_chunks[2],
    );

    let pass_display = if state.show_password {
        state.input_password.clone()
    } else {
        "*".repeat(state.input_password.len())
    };
    f.render_widget(
        Paragraph::new(pass_display).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Password ")
                .border_style(if state.active_field == 3 {
                    active_style
                } else {
                    inactive_style
                }),
        ),
        form_chunks[3],
    );

    let mut hints = vec![Line::from(vec![
        Span::styled("[Tab]", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" next field   "),
        Span::styled("[T]", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" reveal password   "),
        Span::styled("[Enter]", Style::default().add_modifier(Modifier::BOLD)),
        Span::styled(
            " validate connection",
            Style::default().fg(crate::ui::colors::MATRIX_GREEN),
        ),
    ])];

    if let Some(err) = &state.error_msg {
        hints.push(Line::from(""));
        hints.push(Line::from(Span::styled(
            format!(" ERROR: {} ", err),
            Style::default()
                .bg(Color::Red)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )));
    }

    f.render_widget(
        Paragraph::new(hints).alignment(Alignment::Center),
        form_chunks[4],
    );
}

fn render_validating(f: &mut Frame, state: &OnboardingState, area: Rect) {
    let inner = render_shell(
        f,
        area,
        "STEP 3/4",
        "validating provider access",
        "This check is lightweight: verify the endpoint, authenticate the credentials, and confirm the server responds in the expected format.",
    );

    let frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    let spinner = frames[state.spinner_frame % frames.len()];

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6),
            Constraint::Length(7),
            Constraint::Min(0),
        ])
        .split(inner);

    let status = crate::ui::common::render_matrix_box(
        f,
        chunks[0],
        "live connection test",
        crate::ui::colors::MATRIX_GREEN,
    );
    f.render_widget(
        Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                format!("{} Validating server connection...", spinner),
                Style::default()
                    .fg(crate::ui::colors::MATRIX_GREEN)
                    .add_modifier(Modifier::BOLD),
            ))
            .alignment(Alignment::Center),
            Line::from(""),
            Line::from(Span::styled(
                "Checking credentials and downloading only the minimum handshake data.",
                Style::default().fg(Color::White),
            ))
            .alignment(Alignment::Center),
        ]),
        status,
    );

    let checklist = crate::ui::common::render_matrix_box(
        f,
        chunks[1],
        "what is happening right now",
        crate::ui::colors::SOFT_GREEN,
    );
    f.render_widget(
        Paragraph::new(vec![
            Line::from("• resolving the provider endpoint"),
            Line::from("• verifying username/password"),
            Line::from("• confirming the server replies with valid IPTV metadata"),
            Line::from(""),
            Line::from(Span::styled(
                "Typical completion time: 3 to 10 seconds.",
                Style::default().fg(crate::ui::colors::TEXT_SECONDARY),
            )),
        ])
        .wrap(Wrap { trim: true }),
        checklist,
    );
}

fn render_failed(f: &mut Frame, state: &OnboardingState, area: Rect) {
    let inner = render_shell(
        f,
        area,
        "STEP 3/4",
        "provider validation failed",
        "The app could not complete the initial handshake. Review the exact error below, fix the credential set, and try again.",
    );

    let status = crate::ui::common::render_matrix_box(f, inner, "validation report", Color::Red);
    f.render_widget(
        Paragraph::new(vec![
            Line::from(Span::styled(
                "X Failed to authenticate with the server.",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(state.error_msg.as_deref().unwrap_or("Unknown error")),
            Line::from(""),
            Line::from(Span::styled(
                "Press [Enter] to go back and edit the credential set.",
                Style::default().fg(Color::White),
            )),
        ])
        .wrap(Wrap { trim: true }),
        status,
    );
}

fn render_success(f: &mut Frame, _state: &OnboardingState, area: Rect) {
    let inner = render_shell(
        f,
        area,
        "STEP 4/4",
        "playlist source connected successfully",
        "The credential set is valid. The next screen brings you into the app so you can choose what to load first.",
    );

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7),
            Constraint::Length(5),
            Constraint::Min(0),
        ])
        .split(inner);

    let status = crate::ui::common::render_matrix_box(
        f,
        chunks[0],
        "link established",
        crate::ui::colors::MATRIX_GREEN,
    );
    f.render_widget(
        Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                "✓ Connected successfully.",
                Style::default()
                    .fg(crate::ui::colors::MATRIX_GREEN)
                    .add_modifier(Modifier::BOLD),
            ))
            .alignment(Alignment::Center),
            Line::from(""),
            Line::from("Your first playlist source has been stored and is ready for import.")
                .alignment(Alignment::Center),
        ]),
        status,
    );

    let next = crate::ui::common::render_matrix_box(
        f,
        chunks[1],
        "what happens next",
        crate::ui::colors::SOFT_GREEN,
    );
    f.render_widget(
        Paragraph::new(vec![
            Line::from("• You will enter the main app."),
            Line::from("• The first full playlist import can take a bit because channels must be downloaded, decoded, filtered, and indexed."),
            Line::from("• After the first import, cached startup is much faster."),
            Line::from(""),
            Line::from(Span::styled("Press [Enter] to continue into Matrix IPTV.", Style::default().fg(crate::ui::colors::MATRIX_GREEN).add_modifier(Modifier::BOLD))),
        ])
        .wrap(Wrap { trim: true }),
        next,
    );
}

fn render_footer(f: &mut Frame, state: &OnboardingState, area: Rect) {
    let current = match state.step {
        OnboardingStep::Welcome => 1,
        OnboardingStep::HowToGet => 2,
        OnboardingStep::Credentials | OnboardingStep::Validating | OnboardingStep::Failed => 3,
        OnboardingStep::Success => 4,
    };
    let dots = (1..=4)
        .map(|i| if i == current { "●" } else { "○" })
        .collect::<Vec<_>>()
        .join(" ");
    let footer_text = format!(
        " first-time setup  //  step {} of 4  //  {} ",
        current, dots
    );
    let area = Rect {
        x: area.x,
        y: area.y + area.height - 1,
        width: area.width,
        height: 1,
    };
    f.render_widget(
        Paragraph::new(footer_text)
            .fg(crate::ui::colors::TEXT_SECONDARY)
            .alignment(Alignment::Center),
        area,
    );
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
