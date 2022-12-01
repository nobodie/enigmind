use std::{
    fmt::Display,
    io::stdout,
    sync::mpsc::{channel, Receiver, RecvError, Sender},
    thread,
    time::Duration,
};

use anyhow::Result;
use crossterm::event::{self, KeyCode, KeyEvent};
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
    pub fn new(code: &str, crit_index: u8, result: Status) -> Self {
        Self {
            code: code.to_string(),
            crit_index,
            result,
        }
    }
}

#[derive(Clone, Copy)]

pub enum Status {
    True,
    False,
    Unchecked,
}

impl From<Status> for Color {
    fn from(value: Status) -> Self {
        match value {
            Status::True => Color::Green,
            Status::False => Color::Red,
            Status::Unchecked => Color::DarkGray,
        }
    }
}

impl Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Status::True => write!(f, "Right"),
            Status::False => write!(f, "Wrong"),
            Status::Unchecked => write!(f, "     "),
        }
    }
}

pub struct GameData {
    pub criterias: Vec<(String, Status)>,
    pub logs: Vec<GameLog>,
    pub base: u8,
    pub column_count: u8,
}

impl GameData {
    pub fn new(base: u8, column_count: u8) -> Self {
        Self {
            criterias: Vec::new(),
            logs: Vec::new(),
            base,
            column_count,
        }
    }

    pub fn add_criteria(&mut self, crit: &str) {
        self.criterias.push((crit.to_string(), Status::Unchecked));
    }

    pub fn add_log(&mut self, game_log: GameLog) {
        self.criterias[game_log.crit_index as usize].1 = game_log.result;
        self.logs.push(game_log);
    }
}

pub enum InputEvent {
    /// An input event occurred.
    Input(KeyEvent),
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
                    if let event::Event::Key(key) = event::read().unwrap() {
                        event_tx.send(InputEvent::Input(key)).unwrap();
                    }
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

    let general_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3), // Title
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
        .split(general_layout[1]);

    // Render everything from top to bottom
    render_block_with_title(
        frame,
        general_layout[0],
        "",
        format!("Welcome to EnigMind v {}", env!("CARGO_PKG_VERSION")).as_str(),
    );
    render_criterias(frame, gd, game_layout[0]);
    render_tries(frame, gd, game_layout[1]);
    render_block_with_title(
        frame,
        general_layout[2],
        "Command line (/test <code> <crits>) (/bid <solution>) (/quit)",
        "/test 012 12",
    );
}

fn render_block_with_title<B>(frame: &mut Frame<B>, rect: Rect, title: &str, text: &str)
where
    B: Backend,
{
    frame.render_widget(draw_block_with_title(title, text, Color::White), rect);
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
    let crit_count = gd.criterias.len();
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
    for (id, (crit_desc, last_status)) in gd.criterias.iter().enumerate() {
        let line = id / crit_grid_x;
        let col = id % crit_grid_x;

        frame.render_widget(
            draw_block_with_title(
                format!("Criteria {}", id).as_str(),
                crit_desc,
                (*last_status).into(),
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
                .border_style(Style::default() /*.fg(Color::Red)*/)
                .style(Style::default().fg(border_colour))
                .border_type(BorderType::Plain)
                .title(title),
        )
}

pub fn start_ui(gd: &GameData) -> Result<()> {
    // Configure Crossterm backend for tui
    let stdout = stdout();
    crossterm::terminal::enable_raw_mode()?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;
    terminal.hide_cursor()?;

    let tick_rate = Duration::from_millis(200);
    let events = Events::new(tick_rate);

    loop {
        // Render
        terminal.draw(|rect| draw(rect, gd))?;
        // Handle inputs

        // Check if we should exit
        match events.next()? {
            InputEvent::Input(key_event) => {
                if key_event.code == KeyCode::Esc {
                    break;
                }
            }
            InputEvent::Tick => (),
        }
    }

    // Restore the terminal and close application
    terminal.clear()?;
    terminal.show_cursor()?;
    crossterm::terminal::disable_raw_mode()?;

    Ok(())
}

fn main() -> Result<()> {
    //let app = Rc::new(RefCell::new(App::new())); // TODO app is useless for now

    let mut gd = GameData::new(5, 3);
    gd.add_criteria("A is the lowest");
    gd.add_criteria("B is equal to 1");
    gd.add_criteria("Two columns have the same value");

    //gd.add_log(GameLog::new("012", 'B', true));
    gd.add_log(GameLog::new("012", 1, Status::True));
    gd.add_log(GameLog::new("012", 2, Status::False));
    //gd.add_log(GameLog::new("011", 'C', true));

    start_ui(&gd)?;
    Ok(())
}
