use std::io::{self, stdout};
use std::time::{Duration, Instant};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, List, ListItem, Paragraph, Tabs, Wrap,
    },
    Frame, Terminal,
};

// Using mock SDR functionality for demo
use num_complex::Complex32;

/// Application state
pub struct App {
    pub should_quit: bool,
    pub current_tab: usize,
    pub frequency: f64,
    pub sample_rate: f64,
    pub gain: f64,
    pub is_streaming: bool,
    pub status_message: String,
    pub spectrum_data: Vec<f32>,
    pub sample_buffer: Vec<Complex32>,
}

// Temporarily removed SdrConfig for testing

impl App {
    pub fn new() -> Self {
        Self {
            should_quit: false,
            current_tab: 0,
            frequency: 890e6,       // 100 MHz
            sample_rate: 1e6,       // 1 MS/s
            gain: 20.0,             // 30 dB
            is_streaming: false,
            status_message: "DEMO MODE - No USRP hardware detected".to_string(),
            spectrum_data: vec![0.0; 512], // Half of FFT size
            sample_buffer: Vec::new(),
        }
    }

    pub fn on_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
            KeyCode::Tab => {
                self.current_tab = (self.current_tab + 1) % 3;
            }
            KeyCode::Right => {
                self.current_tab = (self.current_tab + 1) % 3;
            }
            KeyCode::Left => {
                self.current_tab = if self.current_tab == 0 { 2 } else { self.current_tab - 1 };
            }
            KeyCode::Char('c') => {
                self.status_message = "MOCK USRP connected (demo mode)".to_string();
            }
            KeyCode::Char('s') => {
                if self.is_streaming {
                    self.stop_streaming();
                } else {
                    self.start_streaming();
                }
            }
            // Parameter adjustments
            KeyCode::Up => self.adjust_parameter(true),
            KeyCode::Down => self.adjust_parameter(false),
            _ => {}
        }
    }

    fn mock_stream_samples(&mut self) {
        // Generate mock IQ samples
        self.sample_buffer.clear();
        for _ in 0..100 { // Limit for display
            // Generate some realistic-looking complex samples
            let phase = rand::random::<f32>() * std::f32::consts::PI * 2.0;
            let magnitude = 0.5 + rand::random::<f32>() * 0.5; // 0.5 to 1.0
            let real = magnitude * phase.cos();
            let imag = magnitude * phase.sin();
            self.sample_buffer.push(Complex32::new(real, imag));
        }
    }

    fn start_streaming(&mut self) {
        self.is_streaming = true;
        self.status_message = "Mock streaming started (demo mode)".to_string();
        self.mock_stream_samples();
    }

    fn stop_streaming(&mut self) {
        self.is_streaming = false;
        self.status_message = "Streaming stopped".to_string();
    }

    fn adjust_parameter(&mut self, increase: bool) {
        let delta = if increase { 1.0 } else { -1.0 };

        match self.current_tab {
            0 => { // Frequency tab
                let step = 1e6; // 1 MHz steps
                self.frequency = (self.frequency + delta * step).max(1e6).min(6e9);
            }
            1 => { // Gain tab
                let step = 1.0; // 1 dB steps
                self.gain = (self.gain + delta * step).max(0.0).min(60.0);
            }
            2 => { // Sample rate tab
                let step = 0.1e6; // 0.1 MS/s steps
                self.sample_rate = (self.sample_rate + delta * step).max(0.1e6).min(10e6);
            }
            _ => {}
        }
    }
}

/// Run the TUI application
pub fn run_tui() -> Result<(), Box<dyn std::error::Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run it
    let mut app = App::new();
    let res = run_app(&mut terminal, &mut app);

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

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> io::Result<()> {
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| ui(f, app))?;

        let timeout = Duration::from_millis(100);
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                app.on_key(key.code);
            }
        }

        // Handle streaming logic
        if app.is_streaming {
            // Continuously update mock data for demo
            simulate_streaming_data(app);
        }

        if app.should_quit {
            break;
        }

        // Throttle updates
        let elapsed = last_tick.elapsed();
        if elapsed < Duration::from_millis(50) {
            std::thread::sleep(Duration::from_millis(50) - elapsed);
        }
        last_tick = Instant::now();
    }

    Ok(())
}

fn simulate_streaming_data(app: &mut App) {
    if !app.is_streaming {
        return;
    }

    // Simulate some spectrum data for demo purposes
    for i in 0..app.spectrum_data.len() {
        let freq = i as f32 / app.spectrum_data.len() as f32;
        let noise = (rand::random::<f32>() - 0.5) * 0.05; // Reduced noise

        // Create multiple signal peaks based on frequency settings
        let center_freq = (app.frequency / 1e9) as f32; // Normalize to 0-1 range (assuming 0-1GHz)
        let signal1 = if (freq - center_freq).abs() < 0.05 { 0.7 } else { 0.0 };
        let signal2 = if (freq - (center_freq + 0.1)).abs() < 0.03 { 0.5 } else { 0.0 };
        let signal3 = if (freq - 0.8).abs() < 0.02 { 0.3 } else { 0.0 }; // Background signal

        let signal = signal1 + signal2 + signal3;
        app.spectrum_data[i] = (signal + noise).max(0.0).min(1.0);
    }

    // Simulate some sample data with realistic IQ characteristics
    app.sample_buffer.clear();
    for _ in 0..20 { // Show more samples
        // Generate IQ samples with some correlation (realistic SDR behavior)
        let phase_noise = rand::random::<f32>() * 0.1;
        let amplitude_noise = rand::random::<f32>() * 0.2;

        let base_amplitude = 0.8 + amplitude_noise;
        let phase = rand::random::<f32>() * std::f32::consts::PI * 2.0 + phase_noise;

        let re = base_amplitude * phase.cos();
        let im = base_amplitude * phase.sin();
        app.sample_buffer.push(Complex32::new(re, im));
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    let size = f.size();

    // Main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Title bar
            Constraint::Min(10),    // Main content
            Constraint::Length(3),  // Status bar
        ])
        .split(size);

    // Title bar with futuristic styling
    let title = Paragraph::new("🛰️  SDR CONTROL TERMINAL  🛰️")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Magenta))
                .style(Style::default().bg(Color::Black)),
        );
    f.render_widget(title, chunks[0]);

    // Main content area
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(chunks[1]);

    // Left panel - Controls
    draw_controls_panel(f, main_chunks[0], app);

    // Right panel - Spectrum and data
    draw_spectrum_panel(f, main_chunks[1], app);

    // Status bar
    draw_status_bar(f, chunks[2], app);
}

fn draw_controls_panel(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Tabs
            Constraint::Min(5),     // Parameters
            Constraint::Length(4),  // Actions
        ])
        .split(area);

    // Tabs
    let titles: Vec<Line> = ["📡 FREQUENCY", "⚡ GAIN", "📊 SAMPLE RATE"]
        .iter()
        .map(|t| Line::from(Span::styled(*t, Style::default().fg(Color::Green))))
        .collect();

    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue))
                .title("CONTROLS")
                .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        )
        .select(app.current_tab)
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));
    f.render_widget(tabs, chunks[0]);

    // Parameter display
    let param_text = match app.current_tab {
        0 => format!("Frequency: {:.3} MHz\n\nUse ↑↓ to adjust\nStep: 1 MHz", app.frequency / 1e6),
        1 => format!("Gain: {:.1} dB\n\nUse ↑↓ to adjust\nStep: 1 dB", app.gain),
        2 => format!("Sample Rate: {:.1} MS/s\n\nUse ↑↓ to adjust\nStep: 0.1 MS/s", app.sample_rate / 1e6),
        _ => "Unknown parameter".to_string(),
    };

    let params = Paragraph::new(param_text)
        .style(Style::default().fg(Color::White))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Green))
                .title("PARAMETER")
                .title_style(Style::default().fg(Color::Green)),
        )
        .wrap(Wrap { trim: true });
    f.render_widget(params, chunks[1]);

    // Action buttons
    let streaming_action = format!(" [S] {} Streaming ", if app.is_streaming { "Stop" } else { "Start" });
    let actions = vec![
        " [C] Connect USRP ".to_string(),
        streaming_action,
        " [Q] Quit ".to_string(),
    ];

    let action_items: Vec<ListItem> = actions
        .iter()
        .map(|action| {
            ListItem::new(Line::from(vec![Span::styled(
                action.clone(),
                Style::default().fg(if action.contains('S') {
                    if app.is_streaming { Color::Green } else { Color::Yellow }
                } else {
                    Color::Cyan
                }),
            )]))
        })
        .collect();

    let actions_list = List::new(action_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Magenta))
                .title("ACTIONS")
                .title_style(Style::default().fg(Color::Magenta)),
        );
    f.render_widget(actions_list, chunks[2]);
}

fn draw_spectrum_panel(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(area);

    // Spectrum display (simplified)
    let spectrum_text = if app.is_streaming {
        let mut display = String::new();
        display.push_str("SPECTRUM ANALYSIS\n\n");

        // Simple ASCII spectrum visualization
        for (i, &power) in app.spectrum_data.iter().enumerate() {
            if i % 16 == 0 { // Show every 16th point for readability
                let bar_len = (power * 20.0) as usize;
                let bar = "█".repeat(bar_len);
                display.push_str(&format!("{:.1}: {}\n", i as f32 / app.spectrum_data.len() as f32, bar));
            }
        }
        display
    } else {
        "SPECTRUM ANALYSIS\n\nNot streaming...\nPress 'S' to start".to_string()
    };

    let spectrum = Paragraph::new(spectrum_text)
        .style(Style::default().fg(Color::Green))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title("SPECTRUM")
                .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        )
        .wrap(Wrap { trim: true });
    f.render_widget(spectrum, chunks[0]);

    // Sample data display
    let sample_text = if !app.sample_buffer.is_empty() {
        let mut display = String::new();
        display.push_str("RECENT SAMPLES\n\n");

        for (i, sample) in app.sample_buffer.iter().take(5).enumerate() {
            display.push_str(&format!("{:2}: {:.4} + j{:.4}\n", i, sample.re, sample.im));
        }

        // Basic stats
        let total_power: f32 = app.sample_buffer.iter().map(|s| s.norm_sqr()).sum();
        let avg_power = total_power / app.sample_buffer.len() as f32;
        display.push_str(&format!("\nAvg Power: {:.6}", avg_power));

        display
    } else {
        "RECENT SAMPLES\n\nNo data available".to_string()
    };

    let samples = Paragraph::new(sample_text)
        .style(Style::default().fg(Color::Yellow))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow))
                .title("SAMPLES")
                .title_style(Style::default().fg(Color::Yellow)),
        )
        .wrap(Wrap { trim: true });
    f.render_widget(samples, chunks[1]);
}

fn draw_status_bar(f: &mut Frame, area: Rect, app: &App) {
    let status = format!(
        " MODE: DEMO | Streaming: {} | {}",
        if app.is_streaming { "ACTIVE" } else { "INACTIVE" },
        app.status_message
    );

    let status_bar = Paragraph::new(status)
        .style(Style::default().fg(Color::White).bg(Color::Blue))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(status_bar, area);
}
