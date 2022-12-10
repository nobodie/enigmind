use std::ops::Deref;

use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

use crate::game_data::GameData;

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
                Constraint::Min(10),                                           // tries
                Constraint::Length(2 + 1 + gd.game.configuration.base as u16), // strikes
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
    render_strikes(frame, gd, tries_strikes_layout[1]);

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

fn render_strikes<B>(frame: &mut Frame<B>, gd: &GameData, rect: Rect)
where
    B: Backend,
{
    frame.render_widget(draw_strikes(gd), rect);
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

fn draw_strikes(gd: &GameData) -> Table {
    let mut rows = Vec::new();

    for row in gd.striked.iter() {
        let mut columns: Vec<Cell> = Vec::new();

        for (value, _striked) in row.iter() {
            let style = match _striked {
                true => Style::default()
                    .bg(Color::Red)
                    .add_modifier(Modifier::CROSSED_OUT),
                false => Style::default().bg(Color::Green),
            };
            columns.push(Cell::from(Span::styled(value.to_string(), style)));
        }

        rows.push(Row::new(columns));
    }

    let header: Vec<Cell> = gd
        .game
        .configuration
        .get_all_columns()
        .iter()
        .map(|col| Cell::from(Span::styled(col.to_string(), Style::default())))
        .collect();

    let constrains = vec![Constraint::Length(1); gd.game.configuration.column_count as usize];

    Table::new(rows)
        .header(Row::new(header))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default())
                .style(Style::default())
                .border_type(BorderType::Plain)
                .title("Strikes"),
        )
        .widths(&constrains)
        .column_spacing(0)
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
