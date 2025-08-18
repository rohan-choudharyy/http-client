use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame, Terminal,
};

use std::io;

use crate::headers::add_headers_to_request;
use crate::json::{pretty_print_json_safe, validate_json};

#[derive(Debug, Clone)]
pub enum HttpMethodType {
    Get,
    Post,
    Put,
    Delete,
}

impl std::fmt::Display for HttpMethodType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpMethodType::Get => write!(f, "GET"),
            HttpMethodType::Post => write!(f, "POST"),
            HttpMethodType::Put => write!(f, "PUT"),
            HttpMethodType::Delete => write!(f, "DELETE"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: HttpMethodType,
    pub url: String,
    pub headers: Vec<String>,
    pub body: Option<String>,
    pub is_json: bool,
}

#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status: u16,
    pub status_text: String,
    pub headers: Vec<(String, String)>,
    pub body: String,
    pub duration_ms: u64,
}

pub enum InputMode {
    Normal,
    EditingUrl,
    EditingHeaders,
    EditingBody,
}

pub enum ActivePanel {
    Request,
    Response,
    History,
}

pub struct App {
    pub should_quit: bool,
    pub input_mode: InputMode,
    pub active_panel: ActivePanel,

    pub method_index: usize,
    pub url: String,
    pub headers_input: String,
    pub body_input: String,
    pub is_json_body: bool,

    pub current_response: Option<HttpResponse>,
    pub history_state: ListState,
    pub status_message: String,
    pub request_history: Vec<(HttpRequest, Option<HttpResponse>)>,

    pub url_cursor_position: usize,
    pub headers_cursor_position: usize,
    pub body_cursor_position: usize,
}

impl Default for App {
    fn default() -> Self {
        let mut history_state = ListState::default();
        history_state.select(Some(0));

        App {
            should_quit: false,
            input_mode: InputMode::Normal,
            active_panel: ActivePanel::Request,
            method_index: 0,
            url: "https://httpbin.org/get".to_string(),
            headers_input: String::new(),
            body_input: String::new(),
            is_json_body: false,
            current_response: None,
            status_message: "Ready".to_string(),
            request_history: Vec::new(),
            history_state,
            url_cursor_position: 0,
            headers_cursor_position: 0,
            body_cursor_position: 0,
        }
    }
}

impl App {
    pub fn get_methods() -> Vec<HttpMethodType> {
        vec![HttpMethodType::Get, HttpMethodType::Post, HttpMethodType::Put, HttpMethodType::Delete]
    }

    pub fn current_method(&self) -> HttpMethodType {
        Self::get_methods()[self.method_index].clone()
    }

    pub fn next_method(&mut self) {
        let methods = Self::get_methods();
        self.method_index = (self.method_index + 1) % methods.len();
    }

    pub fn previous_method(&mut self) {
        let methods = Self::get_methods();
        self.method_index = if self.method_index == 0 {
            methods.len() - 1
        } else {
            self.method_index - 1
        };
    }

    pub async fn send_request(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let start = std::time::Instant::now();

        self.status_message = "Sending request...".to_string();

        let headers: Vec<String> = self.headers_input
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|s| s.to_string())
            .collect();

        let mut request = match self.current_method() {
            HttpMethodType::Get => client.get(&self.url),
            HttpMethodType::Post => client.post(&self.url),
            HttpMethodType::Put => client.put(&self.url),
            HttpMethodType::Delete => client.delete(&self.url),
        };

        request = add_headers_to_request(request, &headers)?;

        if !self.body_input.trim().is_empty() {
            if self.is_json_body {
                validate_json(&self.body_input)?;
                request = request
                    .header("Content-Type", "application/json")
                    .body(self.body_input.clone());
            } else {
                request = request.body(self.body_input.clone());
            }
        }

        let response = request.send().await?;
        let duration = start.elapsed();

        let status = response.status().as_u16();
        let status_text = response.status().to_string();
        let response_headers: Vec<(String, String)> = response
            .headers()
            .iter()
            .map(|(name, value)| {
                (name.to_string(), value.to_str().unwrap_or("").to_string())
            })
            .collect();

        let body = response.text().await?;
        let pretty_body = if body.trim().starts_with('{') || body.trim().starts_with('[') {
            pretty_print_json_safe(&body)
        } else {
            body
        };

        let http_response = HttpResponse {
            status,
            status_text,
            headers: response_headers,
            body: pretty_body,
            duration_ms: duration.as_millis() as u64,
        };

        let http_request = HttpRequest {
            method: self.current_method(),
            url: self.url.clone(),
            headers,
            body: if self.body_input.trim().is_empty() { None } else { Some(self.body_input.clone()) },
            is_json: self.is_json_body,
        };

        self.request_history.push((http_request, Some(http_response.clone())));
        self.current_response = Some(http_response);
        self.active_panel = ActivePanel::Response;
        self.status_message = format!("Request completed in {}ms", duration.as_millis());

        Ok(())
    }
}

pub fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(1), // Status bar
            Constraint::Min(0),    // Main content
        ])
        .split(f.size());
    
    // Status bar
    let status_style = Style::default().fg(Color::Blue);
    let status_text = format!(" Status: {} | Press 'q' to quit, 'h' for help", app.status_message);
    let status = Paragraph::new(status_text).style(status_style);
    f.render_widget(status, chunks[0]);
    
    // Main content split
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50), // Request panel
            Constraint::Percentage(50), // Response/History panel
        ])
        .split(chunks[1]);
    
    // Always draw request panel
    draw_request_panel(f, app, main_chunks[0]);
    
    // Draw right panel based on active panel
    match app.active_panel {
        ActivePanel::Request => draw_request_panel(f, app, main_chunks[0]),
        ActivePanel::Response => draw_response_panel(f, app, main_chunks[1]),
        ActivePanel::History => draw_history_panel(f, app, main_chunks[1]),
    }
}

fn draw_request_panel(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Method + URL
            Constraint::Length(5), // Headers
            Constraint::Min(0),    // Body
        ])
        .split(area);
    
    // Method + URL
    let url_style = match app.input_mode {
        InputMode::EditingUrl => Style::default().fg(Color::Green),
        _ => Style::default().fg(Color::White),
    };
    
    let method_url_text = format!("{} {}", app.current_method(), app.url);
    let method_url = Paragraph::new(method_url_text)
        .style(if matches!(app.input_mode, InputMode::EditingUrl) { url_style } else { Style::default() })
        .block(Block::default().borders(Borders::ALL).title("Request"));
    f.render_widget(method_url, chunks[0]);
    
    // Headers
    let headers_style = match app.input_mode {
        InputMode::EditingHeaders => Style::default().fg(Color::Green),
        _ => Style::default(),
    };
    let headers = Paragraph::new(app.headers_input.as_str())
        .style(headers_style)
        .block(Block::default().borders(Borders::ALL).title("Headers"))
        .wrap(Wrap { trim: true });
    f.render_widget(headers, chunks[1]);
    
    // Body
    let body_style = match app.input_mode {
        InputMode::EditingBody => Style::default().fg(Color::Green),
        _ => Style::default(),
    };
    let body_title = if app.is_json_body { "Body (JSON)" } else { "Body" };
    let body = Paragraph::new(app.body_input.as_str())
        .style(body_style)
        .block(Block::default().borders(Borders::ALL).title(body_title))
        .wrap(Wrap { trim: true });
    f.render_widget(body, chunks[2]);
}

fn draw_response_panel(f: &mut Frame, app: &App, area: Rect) {
    match &app.current_response {
        Some(response) => {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3), // Status
                    Constraint::Length(6), // Headers
                    Constraint::Min(0),    // Body
                ])
                .split(area);
            
            // Status
            let status_color = if response.status >= 200 && response.status < 300 {
                Color::Green
            } else if response.status >= 400 {
                Color::Red
            } else {
                Color::Yellow
            };
            
            let status_text = format!("{} {} ({}ms)", response.status, response.status_text, response.duration_ms);
            let status = Paragraph::new(status_text)
                .style(Style::default().fg(status_color))
                .block(Block::default().borders(Borders::ALL).title("Response Status"));
            f.render_widget(status, chunks[0]);
            
            // Headers
            let header_lines: Vec<Line> = response.headers.iter()
                .take(4) // Limit to first 4 headers
                .map(|(name, value)| {
                    Line::from(vec![
                        Span::styled(name, Style::default().fg(Color::Cyan)),
                        Span::raw(": "),
                        Span::raw(value),
                    ])
                })
                .collect();
            
            let headers = Paragraph::new(Text::from(header_lines))
                .block(Block::default().borders(Borders::ALL).title("Headers"))
                .wrap(Wrap { trim: true });
            f.render_widget(headers, chunks[1]);
            
            // Body
            let body = Paragraph::new(response.body.as_str())
                .block(Block::default().borders(Borders::ALL).title("Response Body"))
                .wrap(Wrap { trim: true });
            f.render_widget(body, chunks[2]);
        }
        None => {
            let placeholder = Paragraph::new("No response yet\n\nPress Enter to send request")
                .block(Block::default().borders(Borders::ALL).title("Response"))
                .style(Style::default().fg(Color::Gray));
            f.render_widget(placeholder, area);
        }
    }
}

fn draw_history_panel(f: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .request_history
        .iter()
        .enumerate()
        .map(|(i, (req, resp))| {
            let status = match resp {
                Some(r) => format!("{}", r.status),
                None => "...".to_string(),
            };
            ListItem::new(format!("{}: {} {} [{}]", i + 1, req.method, req.url, status))
        })
        .collect();
    
    let history = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("History"))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));
    
    f.render_stateful_widget(history, area, &mut app.history_state.clone());
}

pub async fn run_tui() -> Result<(), Box<dyn std::error::Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    
    // Create app and run
    let mut app = App::default();
    let res = run_app(&mut terminal, &mut app).await;
    
    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    
    if let Err(err) = res {
        println!("{:?}", err);
    }
    
    Ok(())
}

async fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        terminal.draw(|f| ui(f, app))?;
        
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match app.input_mode {
                    InputMode::Normal => match key.code {
                        KeyCode::Char('q') => {
                            app.should_quit = true;
                            break;
                        }
                        KeyCode::Char('u') => app.input_mode = InputMode::EditingUrl,
                        KeyCode::Char('h') => app.input_mode = InputMode::EditingHeaders,
                        KeyCode::Char('b') => app.input_mode = InputMode::EditingBody,
                        KeyCode::Char('j') => app.is_json_body = !app.is_json_body,
                        KeyCode::Char('m') => app.next_method(),
                        KeyCode::Char('M') => app.previous_method(),
                        KeyCode::Enter => {
                            if let Err(e) = app.send_request().await {
                                app.status_message = format!("Error: {}", e);
                            }
                        }
                        KeyCode::Tab => {
                            app.active_panel = match app.active_panel {
                                ActivePanel::Request => ActivePanel::Response,
                                ActivePanel::Response => ActivePanel::History,
                                ActivePanel::History => ActivePanel::Request,
                            };
                        }
                        _ => {}
                    },
                    InputMode::EditingUrl => match key.code {
                        KeyCode::Esc => app.input_mode = InputMode::Normal,
                        KeyCode::Enter => app.input_mode = InputMode::Normal,
                        KeyCode::Char(c) => app.url.push(c),
                        KeyCode::Backspace => {
                            app.url.pop();
                        }
                        _ => {}
                    },
                    InputMode::EditingHeaders => match key.code {
                        KeyCode::Esc => app.input_mode = InputMode::Normal,
                        KeyCode::Char(c) => app.headers_input.push(c),
                        KeyCode::Backspace => {
                            app.headers_input.pop();
                        }
                        KeyCode::Enter => app.headers_input.push('\n'),
                        _ => {}
                    },
                    InputMode::EditingBody => match key.code {
                        KeyCode::Esc => app.input_mode = InputMode::Normal,
                        KeyCode::Char(c) => app.body_input.push(c),
                        KeyCode::Backspace => {
                            app.body_input.pop();
                        }
                        KeyCode::Enter => app.body_input.push('\n'),
                        _ => {}
                    },
                }
            }
        }
        
        if app.should_quit {
            break;
        }
    }
    
    Ok(())
}