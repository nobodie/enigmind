use std::{
    fmt::Display,
    io::{stdout, Write},
    sync::mpsc::{channel, Receiver, RecvError, Sender},
    thread,
    time::Duration,
};

use anyhow::Result;
use crossterm::{
    event::{self, EnableMouseCapture, KeyCode, KeyEvent, MouseButton},
    ExecutableCommand,
};
use enigmind_lib::setup::{generate_game, Game};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Table},
    Frame, Terminal,
};

pub struct GameLog {
    code: String,
    crit_index: u8,
    result: Status,
}

impl GameLog {
    pub fn new(code: &str, crit_index: u8, res: bool) -> Self {
        Self {
            code: code.to_string(),
            crit_index,
            result: Status(Some(res)),
        }
    }
}

#[derive(Clone, Copy)]

pub struct Status(Option<bool>);

impl From<Status> for Color {
    fn from(value: Status) -> Self {
        match value.0 {
            None => Color::DarkGray,
            Some(true) => Color::Green,
            Some(false) => Color::Red,
        }
    }
}

impl Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            None => write!(f, "     "),
            Some(true) => write!(f, "Right"),
            Some(false) => write!(f, "Wrong"),
        }
    }
}

pub struct GameData {
    pub game: Game,
    pub logs: Vec<GameLog>,
    pub command_line: String,
    pub last_command_line: String,
    pub command_result: Option<bool>,
    pub quit: bool,
}

impl GameData {
    pub fn new(game: Game) -> Self {
        Self {
            game,
            logs: Vec::new(),
            command_line: String::new(),
            last_command_line: String::new(),
            command_result: None,
            quit: false,
        }
    }

    pub fn process_commands(&mut self) {
        self.command_result = Some(false);
        self.last_command_line = self.command_line.clone();

        if self.command_line.starts_with("/quit") {
            self.process_quit_command();
        } else if self.command_line.starts_with("/test") {
            self.process_test_command();
        } else if self.command_line.starts_with("/bid") {
            self.process_bid_command();
        }
    }

    fn process_quit_command(&mut self) {
        self.command_line = String::new();
        self.command_result = Some(true);
        self.quit = true;
    }

    fn process_test_command(&mut self) {
        self.command_result = Some(false);
        let mut args = self.command_line.split(' ');
        args.next();
        let code_str = args.next().unwrap_or("");
        let criterias = args.next().unwrap_or("");
        if code_str.is_empty() || criterias.is_empty() {
            return;
        }
        let code = code_str.to_string().into();
        if !self.game.is_solution_compatible(&code) {
            return;
        }
        for crit in criterias.chars() {
            if !crit.is_numeric() {
                return;
            }

            let num = crit.to_digit(10);

            match num {
                Some(n) => {
                    if n as usize >= self.game.criterias.len() {
                        return;
                    }
                }
                None => return,
            };
        }
        for crit in criterias.chars() {
            let crit_index = crit.to_digit(10).unwrap();

            let res = self.game.criterias[crit_index as usize]
                .verif
                .rule
                .evaluate(code.clone())
                .unwrap();

            self.logs
                .push(GameLog::new(code_str, crit_index as u8, res));
        }
        self.command_result = Some(true);
        self.command_line = String::new();
    }

    fn process_bid_command(&mut self) {
        self.command_result = Some(false);
        let mut args = self.command_line.split(' ');
        args.next();
        let solution_str = args.next().unwrap_or("");
        if solution_str.is_empty() {
            return;
        }
        let solution = solution_str.to_string().into();
        if !self.game.is_solution_compatible(&solution) {
            return;
        }

        self.command_result = Some(self.game.code == solution);
        self.command_line = String::new();
    }
}

pub enum InputEvent {
    /// An input event occurred.
    Input(KeyEvent),
    ///
    LeftClick(u16, u16),
    /// An tick event occurred.
    Tick,
}

pub struct Events {
    rx: Receiver<InputEvent>,
    // Need to be kept around to prevent disposing the sender side.
    _tx: Sender<InputEvent>,
}

impl Events {
    pub fn new(tick_rate: Duration) -> Events {
        let (tx, rx) = channel();

        let event_tx = tx.clone(); // the thread::spawn own event_tx

        thread::spawn(move || {
            loop {
                // poll for tick rate duration, if no event, sent tick event.
                if crossterm::event::poll(tick_rate).unwrap() {
                    match event::read().unwrap() {
                        event::Event::Key(key) => event_tx.send(InputEvent::Input(key)).unwrap(),
                        event::Event::Mouse(mouse_event) => {
                            if let event::MouseEventKind::Down(MouseButton::Left) = mouse_event.kind
                            {
                                dbg!(mouse_event);
                                event_tx
                                    .send(InputEvent::LeftClick(
                                        mouse_event.column,
                                        mouse_event.row,
                                    ))
                                    .unwrap();
                            }
                        }
                        _ => (),
                    };
                }
                event_tx.send(InputEvent::Tick).unwrap();
            }
        });

        Events { rx, _tx: tx }
    }

    /// Attempts to read an event.
    /// This function block the current thread.
    pub fn next(&self) -> Result<InputEvent, RecvError> {
        self.rx.recv()
    }
}

pub fn draw<B>(frame: &mut Frame<B>, gd: &GameData)
where
    B: Backend,
{
    let size = frame.size();

    let solution_vert_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(50),
                Constraint::Length(3),
                Constraint::Percentage(50),
            ]
            .as_ref(),
        )
        .split(size);

    let solution_horiz_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Ratio(1, 2),
                Constraint::Length(20),
                Constraint::Ratio(1, 2),
            ]
            .as_ref(),
        )
        .split(solution_vert_layout[1]);

    let general_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3), // Title
                Constraint::Length(3), // Rules
                Constraint::Min(10),   // Criterias + tries
                Constraint::Length(3), // Command line
            ]
            .as_ref(),
        )
        .split(size);

    let game_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Min(10),                   // Criterias
                Constraint::Length(2 + 2 + 6 + 4 + 5), // logs
            ]
            .as_ref(),
        )
        .split(general_layout[2]);

    let tries_strikes_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Min(10),                                       // tries
                Constraint::Length(2 + gd.game.configuration.base as u16), // strikes
            ]
            .as_ref(),
        )
        .split(game_layout[1]);

    // Render everything from top to bottom
    render_block_with_title(
        frame,
        general_layout[0],
        "",
        format!("Welcome to EnigMind v {}", env!("CARGO_PKG_VERSION")).as_str(),
        Color::White,
    );

    render_block_with_title(
        frame,
        general_layout[1],
        "Rules",
        format!(
            "You must find a code of {} digits between 0 and {}",
            gd.game.configuration.column_count,
            gd.game.configuration.base - 1
        )
        .as_str(),
        Color::White,
    );

    render_criterias(frame, gd, game_layout[0]);

    render_tries(frame, gd, tries_strikes_layout[0]);
    render_block_with_title(
        frame,
        tries_strikes_layout[1],
        "Strikes",
        "444\n333\n222\n111\n000",
        Color::White,
    );

    let command_line_color = match gd.command_result {
        None => Color::DarkGray,
        Some(true) => Color::Green,
        Some(false) => Color::Red,
    };

    render_block_with_title(
        frame,
        general_layout[3],
        "Command line (/test <code> <crits>) (/bid <solution>) (/quit)",
        &gd.command_line,
        command_line_color,
    );

    clear_block(frame, solution_horiz_layout[1]);

    render_block_with_title(
        frame,
        solution_horiz_layout[1],
        "Solution",
        "Bravo",
        Color::Green,
    );
}

fn render_block_with_title<B>(frame: &mut Frame<B>, rect: Rect, title: &str, text: &str, col: Color)
where
    B: Backend,
{
    frame.render_widget(draw_block_with_title(title, text, col), rect);
}

fn clear_block<B>(frame: &mut Frame<B>, rect: Rect)
where
    B: Backend,
{
    let line: String = (0..rect.width).map(|_| " ").collect();
    let filler = vec![line; rect.height as usize].join("\n");

    let p = Paragraph::new(filler).style(Style::default());

    frame.render_widget(p, rect);
}

fn render_tries<B>(frame: &mut Frame<B>, gd: &GameData, rect: Rect)
where
    B: Backend,
{
    frame.render_widget(draw_tries(gd), rect);
}

fn render_criterias<B>(frame: &mut Frame<B>, gd: &GameData, rect: Rect)
where
    B: Backend,
{
    let crit_count = gd.game.criterias.len();
    let crit_grid_x = ((crit_count as f64 - 1.0).sqrt() as usize) + 1;
    let crit_grid_y = ((crit_count - 1) / crit_grid_x) + 1;
    let mut constraints_y = Vec::new();
    for _ in 0..crit_grid_y {
        constraints_y.push(Constraint::Ratio(1, crit_grid_y as u32));
    }
    let mut constraints_x = Vec::new();
    for _ in 0..crit_grid_x {
        constraints_x.push(Constraint::Ratio(1, crit_grid_x as u32));
    }
    let mut crit_array = Vec::new();
    for crit_line in Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints_y)
        .split(rect)
    {
        let crit_column = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(constraints_x.clone())
            .split(crit_line);

        crit_array.push(crit_column);
    }
    for (id, crit) in gd.game.criterias.iter().enumerate() {
        let line = id / crit_grid_x;
        let col = id % crit_grid_x;

        frame.render_widget(
            draw_block_with_title(
                format!("Criteria {}", id).as_str(),
                crit.description.as_str(),
                Color::Gray,
            ),
            crit_array[line][col],
        );
    }
}

fn draw_tries(gd: &GameData) -> Table {
    let mut rows = Vec::new();

    for log in gd.logs.iter() {
        rows.push(Row::new(vec![
            Cell::from(Span::styled(
                log.code.as_str(),
                Style::default().fg(log.result.into()),
            )),
            Cell::from(Span::styled(
                log.crit_index.to_string(),
                Style::default().fg(log.result.into()),
            )),
            Cell::from(Span::styled(
                log.result.to_string(),
                Style::default()
                    .fg(log.result.into())
                    .add_modifier(Modifier::REVERSED),
            )),
        ]));
    }

    let header = Row::new(vec![
        Cell::from(Span::styled("Code", Style::default())),
        Cell::from(Span::styled("Crit", Style::default())),
        Cell::from(Span::styled("Result", Style::default())),
    ]);

    Table::new(rows)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default())
                .style(Style::default())
                .border_type(BorderType::Plain)
                .title("Tries"),
        )
        .widths(&[
            Constraint::Length(5),
            Constraint::Length(4),
            Constraint::Length(6),
        ])
        .column_spacing(1)
}

fn draw_block_with_title<'a>(title: &'a str, text: &'a str, border_colour: Color) -> Paragraph<'a> {
    Paragraph::new(text)
        .style(Style::default().fg(Color::LightCyan))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default())
                .style(
                    Style::default()
                        .fg(border_colour)
                        .add_modifier(Modifier::empty()),
                )
                .border_type(BorderType::Plain)
                .title(title),
        )
}

pub fn start_ui(gd: &mut GameData) -> Result<()> {
    // Configure Crossterm backend for tui
    let mut stdout = stdout();
    stdout.execute(EnableMouseCapture)?;
    stdout.flush()?;
    crossterm::terminal::enable_raw_mode()?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;
    //terminal.hide_cursor()?;
    terminal.backend_mut().execute(EnableMouseCapture)?;

    let tick_rate = Duration::from_millis(200);
    let events = Events::new(tick_rate);

    while !gd.quit {
        // Render
        terminal.draw(|rect| draw(rect, gd))?;
        // Handle inputs

        // Check if we should exit
        if let InputEvent::Input(key_event) = events.next()? {
            match key_event.code {
                KeyCode::Esc => break,
                KeyCode::Char(c) => {
                    gd.command_line.push(c);
                    gd.command_result = None
                }
                KeyCode::Backspace => {
                    gd.command_line.pop();
                    gd.command_result = None;
                }
                KeyCode::Up => gd.command_line = gd.last_command_line.clone(),
                KeyCode::Enter => gd.process_commands(),
                _ => (),
            }
        }
    }

    // Restore the terminal and close application
    terminal.clear()?;
    terminal.show_cursor()?;
    crossterm::terminal::disable_raw_mode()?;

    Ok(())
}

fn main() -> Result<()> {
    let game = generate_game(5, 3, 10).unwrap();

    let mut gd = GameData::new(game);

    start_ui(&mut gd)?;
    Ok(())
}
