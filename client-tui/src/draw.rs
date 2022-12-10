use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

use crate::game_data::{GameData, Status};

fn centered(r: Rect, size: (u16, u16)) -> Rect {
    let solution_vert_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length((r.height - size.1) / 2),
                Constraint::Length(size.1),
                Constraint::Length((r.height - size.1) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Length((r.width - size.0) / 2),
                Constraint::Length(size.0),
                Constraint::Length((r.width - size.0) / 2),
            ]
            .as_ref(),
        )
        .split(solution_vert_layout[1])[1]
}

pub fn draw<B>(frame: &mut Frame<B>, gd: &GameData)
where
    B: Backend,
{
    let size = frame.size();

    let centered_layout = centered(size, (19, 4));

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

    let command_line_color = match gd.command_status {
        Status::None => Color::DarkGray,
        Status::Valid => Color::Green,
        Status::Error => Color::Red,
    };

    render_block_with_title(
        frame,
        general_layout[3],
        "Command line (/test <code> <crits>) (/bid <solution>) (/quit)",
        &gd.command_line,
        command_line_color,
    );

    if let Some(val) = gd.solution {
        let (color, mut text) = match val {
            true => (Color::Green, "Well done!".to_string()),
            false => (Color::Red, "You failed!".to_string()),
        };

        text.push_str("\nEnter to continue");

        clear_block(frame, centered_layout);

        render_block_with_title(frame, centered_layout, "Solution", &text, color);
    }
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

    let header = gd
        .game
        .configuration
        .get_all_columns()
        .into_iter()
        .map(|col| Cell::from(Span::styled(col.to_string(), Style::default())));

    let constrains = vec![Constraint::Length(1); gd.game.configuration.column_count as usize];

    let table = Table::new(rows)
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
        .column_spacing(0);

    frame.render_widget(table, rect);
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
                format!("Criteria {id}").as_str(),
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
        let color = match log.result {
            true => Color::Green,
            false => Color::Red,
        };

        let msg = match log.result {
            true => "Right",
            false => "Wrong",
        }
        .to_owned();

        rows.push(Row::new(vec![
            Cell::from(Span::styled(log.code.as_str(), Style::default().fg(color))),
            Cell::from(Span::styled(
                log.crit_index.to_string(),
                Style::default().fg(color),
            )),
            Cell::from(Span::styled(
                msg,
                Style::default().fg(color).add_modifier(Modifier::REVERSED),
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
