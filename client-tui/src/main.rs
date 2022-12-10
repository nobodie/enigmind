mod draw;
mod game_data;
mod input;

use std::{
    io::{stdout, Write},
    time::Duration,
};

use anyhow::Result;
use crossterm::{event::EnableMouseCapture, ExecutableCommand};
use enigmind_lib::setup::generate_game;
use game_data::GameData;
use input::Events;
use tui::{backend::CrosstermBackend, Terminal};

use crate::draw::draw;

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
        gd.handle_events(&events);
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
