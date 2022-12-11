use crossterm::event::{KeyCode, MouseButton};
use enigmind_lib::setup::Game;
use tui::{layout::Rect, style::Color};

use crate::input::{Events, InputEvent};

pub struct GameLog {
    pub code: String,
    pub crit_index: u8,
    pub result: bool,
}

impl GameLog {
    pub fn new(code: &str, crit_index: u8, res: bool) -> Self {
        Self {
            code: code.to_string(),
            crit_index,
            result: res,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Status {
    None,
    Valid,
    Error,
}

impl From<Status> for Color {
    fn from(value: Status) -> Self {
        match value {
            Status::None => Color::DarkGray,
            Status::Valid => Color::Green,
            Status::Error => Color::Red,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ClickAction {
    ToggleStrike(u16, u16),
    ToggleCriteriaRule(u16, u16),
    CloseSolutionWidget,
}

pub struct GameData {
    pub game: Game,
    pub logs: Vec<GameLog>,
    pub command_line: String,
    pub last_command_line: String,
    pub command_status: Status,
    pub quit: bool,
    pub striked: Vec<Vec<(char, bool)>>,
    pub solution: Option<bool>,
    pub criterias_state: Vec<Vec<bool>>,
    pub click_areas: Vec<(Rect, ClickAction)>,
}

impl GameData {
    pub fn new(game: Game) -> Self {
        let mut striked = Vec::new();
        for i in (0..game.configuration.base).rev() {
            let val = i.to_string().chars().nth(0).unwrap();
            let line = vec![(val, false); game.configuration.column_count as usize];
            striked.push(line);
        }

        let mut criterias_state = Vec::new();

        for crit in game.criterias.iter() {
            criterias_state.push(vec![true; crit.rules.len()]);
        }

        Self {
            game,
            logs: Vec::new(),
            command_line: String::new(),
            last_command_line: String::new(),
            command_status: Status::None,
            quit: false,
            striked,
            solution: None,
            click_areas: Vec::new(),
            criterias_state,
        }
    }

    pub fn handle_events(&mut self, events: &Events) {
        match events.next().unwrap() {
            InputEvent::Input(key_event) => match key_event.code {
                KeyCode::Esc => self.quit = true,
                KeyCode::Char(c) => {
                    self.command_line.push(c);
                    self.command_status = Status::None
                }
                KeyCode::Backspace => {
                    self.command_line.pop();
                    self.command_status = Status::None;
                }
                KeyCode::Up => self.command_line = self.last_command_line.clone(),
                KeyCode::Enter => {
                    self.solution = None;
                    self.process_commands()
                }
                _ => (),
            },
            InputEvent::Click(mb, x, y) => self.process_click(mb, x, y),
            InputEvent::Tick => (),
        };
    }
}

impl GameData {
    fn process_click(&mut self, mb: MouseButton, x: u16, y: u16) {
        if mb == MouseButton::Left {
            for (rect, action) in self.click_areas.clone().into_iter().rev() {
                if x >= rect.x && x < rect.x + rect.width && y >= rect.y && y < rect.y + rect.height
                {
                    self.process_click_action(action);
                    return;
                }
            }
        }
    }

    fn process_click_action(&mut self, action: ClickAction) {
        match action {
            ClickAction::ToggleStrike(x, y) => {
                self.striked[y as usize][x as usize].1 ^= true;
            }
            ClickAction::ToggleCriteriaRule(crit, rule) => {
                self.criterias_state[crit as usize][rule as usize] ^= true
            }
            ClickAction::CloseSolutionWidget => self.solution = None,
        }
    }

    fn process_commands(&mut self) {
        self.last_command_line = self.command_line.clone();

        let command = self.command_line.split(' ').next().unwrap();

        self.command_status = match command {
            "q" => self.process_quit_command(),
            "t" => self.process_test_command(),
            "b" => self.process_bid_command(),
            "s" => self.process_toggle_command(),
            _ => Status::Error,
        };

        if self.command_status == Status::Valid {
            self.command_line.clear();
        }
    }

    fn process_toggle_command(&mut self) -> Status {
        let mut args = self.command_line.split(' ');
        args.next();

        for arg in args.clone() {
            if arg.len() != 2 {
                return Status::Error;
            }

            let column_str = arg.chars().nth(0).unwrap().to_ascii_uppercase();
            let value_str = arg.chars().nth(1).unwrap();

            if !column_str.is_alphabetic() || !value_str.is_numeric() {
                return Status::Error;
            }

            if !self.game.is_column_compatible(column_str) {
                return Status::Error;
            }

            let value = value_str.to_digit(10).unwrap() as u8;
            if !self.game.is_value_compatible(value) {
                return Status::Error;
            }
        }

        for arg in args {
            let column_index = self
                .game
                .to_column_index(arg.chars().nth(0).unwrap().to_ascii_uppercase());
            let value =
                self.striked.len() - 1 - arg.chars().nth(1).unwrap().to_digit(10).unwrap() as usize;

            self.striked[value][column_index as usize].1 ^= true;
        }

        Status::Valid
    }

    fn process_quit_command(&mut self) -> Status {
        self.quit = true;
        Status::Valid
    }

    fn process_test_command(&mut self) -> Status {
        let mut args = self.command_line.split(' ');
        args.next();
        let code_str = args.next().unwrap_or("");
        let criterias = args.next().unwrap_or("");
        if code_str.is_empty() || criterias.is_empty() {
            return Status::Error;
        }
        let code = code_str.to_string().into();
        if !self.game.is_solution_compatible(&code) {
            return Status::Error;
        }
        for crit in criterias.chars() {
            if !crit.is_numeric() {
                return Status::Error;
            }

            let num = crit.to_digit(10);

            match num {
                Some(n) => {
                    if n as usize >= self.game.criterias.len() {
                        return Status::Error;
                    }
                }
                None => return Status::Error,
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

        Status::Valid
    }

    fn process_bid_command(&mut self) -> Status {
        let mut args = self.command_line.split(' ');
        args.next();
        let solution_str = args.next().unwrap_or("");
        if solution_str.is_empty() {
            return Status::Error;
        }
        let solution = solution_str.to_string().into();
        if !self.game.is_solution_compatible(&solution) {
            return Status::Error;
        }

        self.solution = Some(solution == self.game.code);

        Status::Valid
    }
}
