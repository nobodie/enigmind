use std::fmt::Display;

use crossterm::event::KeyCode;
use enigmind_lib::setup::Game;
use tui::style::Color;

use crate::input::{Events, InputEvent};

pub struct GameLog {
    pub code: String,
    pub crit_index: u8,
    pub result: Status,
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
    pub striked: Vec<Vec<(char, bool)>>,
    pub solution: Status,
}

impl GameData {
    pub fn new(game: Game) -> Self {
        let mut striked = Vec::new();
        for i in (0..game.configuration.base).rev() {
            let val = i.to_string().chars().nth(0).unwrap();
            let line = vec![(val, false); game.configuration.column_count as usize];
            striked.push(line);
        }

        Self {
            game,
            logs: Vec::new(),
            command_line: String::new(),
            last_command_line: String::new(),
            command_result: None,
            quit: false,
            striked,
            solution: Status(None),
        }
    }

    pub fn handle_events(&mut self, events: &Events) {
        if let InputEvent::Input(key_event) = events.next().unwrap() {
            match key_event.code {
                KeyCode::Esc => self.quit = true,
                KeyCode::Char(c) => {
                    self.command_line.push(c);
                    self.command_result = None
                }
                KeyCode::Backspace => {
                    self.command_line.pop();
                    self.command_result = None;
                }
                KeyCode::Up => self.command_line = self.last_command_line.clone(),
                KeyCode::Enter => self.process_commands(),
                _ => (),
            }
        }
    }
}

impl GameData {
    fn process_commands(&mut self) {
        self.command_result = Some(false);
        self.last_command_line = self.command_line.clone();

        if self.command_line.starts_with("/quit") {
            self.process_quit_command();
        } else if self.command_line.starts_with("/test") {
            self.process_test_command();
        } else if self.command_line.starts_with("/bid") {
            self.process_bid_command();
        } else if self.command_line.starts_with("/strike") {
            self.process_strike_command(true);
        } else if self.command_line.starts_with("/unstrike") {
            self.process_strike_command(false);
        }
    }

    fn process_strike_command(&mut self, strike: bool) {
        self.command_result = Some(false);
        let mut args = self.command_line.split(' ');
        args.next();

        for arg in args.clone().into_iter() {
            if arg.len() != 2 {
                return;
            }

            let column_str = arg.chars().nth(0).unwrap();
            let value_str = arg.chars().nth(1).unwrap();

            if !column_str.is_alphabetic() || !value_str.is_numeric() {
                return;
            }

            if !self.game.is_column_compatible(column_str) {
                return;
            }

            let value = value_str.to_digit(10).unwrap() as u8;
            if !self.game.is_value_compatible(value) {
                return;
            }
        }

        for arg in args.into_iter() {
            let column_index = self.game.to_column_index(arg.chars().nth(0).unwrap());
            let value =
                self.striked.len() - 1 - arg.chars().nth(1).unwrap().to_digit(10).unwrap() as usize;

            self.striked[value][column_index as usize].1 = strike;
        }

        self.command_result = Some(true);
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
