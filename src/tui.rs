// Citation for use of AI tools:
// Date: 2025-09-05
// Prompts: "Can you integrate a TUI using the ratatui crate for rust?"
// AI Source URL: https://www.anthropic.com/claude/sonnet

use std::io;
use std::time::{Duration, Instant};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, Clear, Gauge, List, ListItem, Paragraph, Wrap,
    },
    Frame, Terminal,
};

use crate::app::{App, AppState, BattleEventType};

pub struct Tui<B: Backend> {
    terminal: Terminal<B>,
    last_tick: Instant,
}

impl<B: Backend> Tui<B> {
    pub fn new(terminal: Terminal<B>) -> Self {
        Self {
            terminal,
            last_tick: Instant::now(),
        }
    }

    pub fn run(&mut self, mut app: App) -> io::Result<()> {
        let mut rng = rand::rng();

        loop {
            self.terminal.draw(|f| Self::render_static(f, &app))?;

            let timeout = Duration::from_millis(app.tick_rate);
            if crossterm::event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char('q') => {
                            app.quit();
                        }
                        KeyCode::Char(' ') => {
                            match app.state {
                                AppState::Setup => app.start_battle(),
                                AppState::Running | AppState::Paused => app.toggle_pause(),
                                AppState::Finished => app.quit(),
                                _ => {}
                            }
                        }
                        KeyCode::Char('s') => {
                            if app.state == AppState::Paused || app.state == AppState::Running {
                                app.step_battle(&mut rng);
                            }
                        }
                        KeyCode::Char('a') => {
                            app.toggle_auto_advance();
                        }
                        KeyCode::Char('r') => {
                            if app.state == AppState::Finished {
                                // Reset the app for a new battle
                                // Note: This would need the original player setup logic
                            }
                        }
                        _ => {}
                    }
                }
            }

            // Auto-advance logic
            if app.auto_advance && app.state == AppState::Running {
                let now = Instant::now();
                if now.duration_since(self.last_tick) >= Duration::from_millis(app.tick_rate) {
                    app.step_battle(&mut rng);
                    self.last_tick = now;
                }
            }

            if app.should_quit() {
                break;
            }
        }

        Ok(())
    }

    fn render_static(f: &mut Frame, app: &App) {
        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(3),      // Title bar
                Constraint::Min(10),        // Main content
                Constraint::Length(3),      // Status bar
            ])
            .split(f.size());

        // Render title
        Self::render_title(f, main_layout[0], app);

        // Render main content based on app state
        match app.state {
            AppState::Setup => Self::render_setup(f, main_layout[1], app),
            AppState::Running | AppState::Paused | AppState::Finished => {
                Self::render_battle(f, main_layout[1], app)
            }
            AppState::Quit => {}
        }

        // Render status bar
        Self::render_status(f, main_layout[2], app);

        // Render help popup if needed
        if app.state == AppState::Setup {
            Self::render_help_popup(f, app);
        }
    }

    fn render_title(f: &mut Frame, area: Rect, app: &App) {
        let title = match app.state {
            AppState::Setup => "Battlers - Setup",
            AppState::Running => "Battlers - Battle in Progress",
            AppState::Paused => "Battlers - Paused",
            AppState::Finished => "Battlers - Battle Finished",
            AppState::Quit => "Battlers - Exiting",
        };

        let title_block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Cyan))
            .title(title);

        let title_paragraph = Paragraph::new(format!("Turn: {} | Players: {}", 
                                                    app.current_turn, 
                                                    app.game.players.len()))
            .block(title_block)
            .alignment(Alignment::Center);

        f.render_widget(title_paragraph, area);
    }

    fn render_setup(f: &mut Frame, area: Rect, app: &App) {
        let setup_text = vec![
            Line::from("Welcome to Battlers!"),
            Line::from(""),
            Line::from(format!("Players loaded: {}", app.game.players.len())),
            Line::from(""),
            Line::from("Controls:"),
            Line::from("  SPACE - Start Battle"),
            Line::from("  q     - Quit"),
            Line::from(""),
            Line::from("Press SPACE to begin the battle simulation."),
        ];

        let setup_paragraph = Paragraph::new(setup_text)
            .block(Block::default().borders(Borders::ALL).title("Setup"))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });

        f.render_widget(setup_paragraph, area);
    }

    fn render_battle(f: &mut Frame, area: Rect, app: &App) {
        let battle_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(40), // Battle arena
                Constraint::Percentage(60), // Info panel
            ])
            .split(area);

        // Render battle arena
        Self::render_arena(f, battle_layout[0], app);

        // Split info panel vertically
        let info_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(50), // Player stats
                Constraint::Percentage(50), // Battle log
            ])
            .split(battle_layout[1]);

        // Render player stats
        Self::render_player_stats(f, info_layout[0], app);

        // Render battle log
        Self::render_battle_log(f, info_layout[1], app);
    }

    fn render_arena(f: &mut Frame, area: Rect, app: &App) {
        let arena_block = Block::default()
            .borders(Borders::ALL)
            .title("Battle Arena");

        let inner_area = arena_block.inner(area);
        f.render_widget(arena_block, area);

        // Draw arena background
        let arena_width = inner_area.width as f32;
        let arena_height = inner_area.height as f32;
        
        // Calculate scale based on player positions
        let (min_x, max_x, min_y, max_y) = app.game.players.iter()
            .fold((60.0f32, 0.0f32, 60.0f32, 0.0f32), |(min_x, max_x, min_y, max_y), player| {
                (
                    min_x.min(player.loc.x),
                    max_x.max(player.loc.x),
                    min_y.min(player.loc.y),
                    max_y.max(player.loc.y),
                )
            });

        let scale_x = if max_x > min_x { (arena_width - 2.0) / (max_x - min_x) } else { 1.0 };
        let scale_y = if max_y > min_y { (arena_height - 2.0) / (max_y - min_y) } else { 1.0 };
        let scale = scale_x.min(scale_y).min(1.0);

        // Draw players on the arena
        for (i, player) in app.game.players.iter().enumerate() {
            let screen_x = ((player.loc.x - min_x) * scale + 1.0) as u16;
            let screen_y = ((player.loc.y - min_y) * scale + 1.0) as u16;
            
            if screen_x < inner_area.width && screen_y < inner_area.height {
                let player_area = Rect {
                    x: inner_area.x + screen_x,
                    y: inner_area.y + screen_y,
                    width: 1,
                    height: 1,
                };

                let player_char = if player.is_dead() { 
                    "✗" 
                } else { 
                    match i {
                        0 => "●",
                        1 => "■",
                        2 => "▲",
                        3 => "♦",
                        _ => "○",
                    }
                };

                let player_color = if player.is_dead() {
                    Color::DarkGray
                } else {
                    match i {
                        0 => Color::Red,
                        1 => Color::Blue,
                        2 => Color::Green,
                        3 => Color::Yellow,
                        _ => Color::Magenta,
                    }
                };

                let player_widget = Paragraph::new(player_char)
                    .style(Style::default().fg(player_color));
                
                f.render_widget(player_widget, player_area);
            }
        }
    }

    fn render_player_stats(f: &mut Frame, area: Rect, app: &App) {
        let stats_block = Block::default()
            .borders(Borders::ALL)
            .title("Player Stats");

        let inner_area = stats_block.inner(area);
        f.render_widget(stats_block, area);

        let mut stats_items = Vec::new();
        for (i, player) in app.game.players.iter().enumerate() {
            let health_percentage = if player.armor.base > 0 {
                (player.armor.curr as f64 / player.armor.base as f64).max(0.0)
            } else {
                0.0
            };

            let status = if player.is_dead() { " [DEAD]" } else { "" };
            
            let player_color = match i {
                0 => Color::Red,
                1 => Color::Blue,
                2 => Color::Green,
                3 => Color::Yellow,
                _ => Color::Magenta,
            };

            let player_info = Line::from(vec![
                Span::styled(
                    format!("{}{}", player.name, status),
                    Style::default().fg(player_color).add_modifier(Modifier::BOLD)
                ),
            ]);

            stats_items.push(ListItem::new(player_info));

            // Health bar
            if inner_area.height > stats_items.len() as u16 * 3 {
                let _health_gauge = Gauge::default()
                    .block(Block::default())
                    .gauge_style(Style::default().fg(Color::Red))
                    .ratio(health_percentage)
                    .label(format!("{}/{}", player.armor.curr, player.armor.base));

                // We'd need to manually position this gauge, which is complex in this context
                // For now, we'll include the health info in the text
                let health_info = Line::from(format!("  Health: {}/{} ({}%)", 
                    player.armor.curr, 
                    player.armor.base, 
                    (health_percentage * 100.0) as u8));
                stats_items.push(ListItem::new(health_info));

                let attack_info = Line::from(format!("  ATK:{} DEF:{} PWR:{} SPD:{} RNG:{}", 
                    player.attack.curr, 
                    player.defense.curr, 
                    player.power.curr,
                    player.speed.curr,
                    player.range.curr));
                stats_items.push(ListItem::new(attack_info));
            }
        }

        let stats_list = List::new(stats_items)
            .style(Style::default().fg(Color::White));

        f.render_widget(stats_list, inner_area);
    }

    fn render_battle_log(f: &mut Frame, area: Rect, app: &App) {
        let log_block = Block::default()
            .borders(Borders::ALL)
            .title("Battle Log");

        let inner_area = log_block.inner(area);
        f.render_widget(log_block, area);

        let log_items: Vec<ListItem> = app
            .get_battle_log()
            .iter()
            .rev()
            .take(inner_area.height as usize)
            .map(|event| {
                let style = match event.event_type {
                    BattleEventType::Hit => Style::default().fg(Color::Red),
                    BattleEventType::Miss => Style::default().fg(Color::Yellow),
                    BattleEventType::Death => Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD),
                    BattleEventType::Movement => Style::default().fg(Color::Cyan),
                    BattleEventType::Info => Style::default().fg(Color::White),
                    BattleEventType::Attack => Style::default().fg(Color::Green),
                };

                let line = Line::from(vec![
                    Span::styled(format!("[{}] ", event.turn), Style::default().fg(Color::DarkGray)),
                    Span::styled(event.message.clone(), style),
                ]);

                ListItem::new(line)
            })
            .collect();

        let log_list = List::new(log_items)
            .style(Style::default().fg(Color::White));

        f.render_widget(log_list, inner_area);
    }

    fn render_status(f: &mut Frame, area: Rect, app: &App) {
        let status_text = match app.state {
            AppState::Setup => "Press SPACE to start | q to quit",
            AppState::Running => {
                if app.auto_advance {
                    "SPACE: Pause | s: Step | a: Toggle Auto | q: Quit [AUTO MODE]"
                } else {
                    "SPACE: Pause | s: Step | a: Toggle Auto | q: Quit"
                }
            },
            AppState::Paused => "SPACE: Resume | s: Step | a: Toggle Auto | q: Quit [PAUSED]",
            AppState::Finished => {
                if let Some(winner) = app.get_winner() {
                    &format!("Winner: {} | SPACE or q: Quit", winner.name)
                } else {
                    "Battle ended inconclusively | SPACE or q: Quit"
                }
            },
            AppState::Quit => "Exiting...",
        };

        let status_block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Yellow));

        let status_paragraph = Paragraph::new(status_text)
            .block(status_block)
            .alignment(Alignment::Center);

        f.render_widget(status_paragraph, area);
    }

    fn render_help_popup(f: &mut Frame, _app: &App) {
        let area = f.size();
        let popup_area = Rect {
            x: area.width / 4,
            y: area.height / 4,
            width: area.width / 2,
            height: area.height / 2,
        };

        f.render_widget(Clear, popup_area);

        let help_text = vec![
            Line::from("Help"),
            Line::from(""),
            Line::from("Controls:"),
            Line::from("  SPACE - Start/Pause/Resume"),
            Line::from("  s     - Single Step"),
            Line::from("  a     - Toggle Auto Mode"),
            Line::from("  q     - Quit"),
            Line::from(""),
            Line::from("Arena Symbols:"),
            Line::from("  ● ■ ▲ ♦ - Living Players"),
            Line::from("  ✗         - Dead Players"),
        ];

        let help_block = Block::default()
            .borders(Borders::ALL)
            .title("Help")
            .style(Style::default().fg(Color::Yellow));

        let help_paragraph = Paragraph::new(help_text)
            .block(help_block)
            .wrap(Wrap { trim: true });

        f.render_widget(help_paragraph, popup_area);
    }
}

pub fn setup_terminal() -> io::Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

pub fn restore_terminal() -> io::Result<()> {
    disable_raw_mode()?;
    execute!(
        io::stdout(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    Ok(())
}
