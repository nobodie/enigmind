use std::{
    io::{self, Write},
    str::FromStr,
    time::Duration,
};

use anyhow::Result;
use enigmind_lib::{code::Code, setup::Game};

use tokio::{
    select,
    time::{sleep, timeout},
};

pub fn read_from_terminal<T>(text: String, min: T, max: T) -> T
where
    T: PartialOrd + FromStr,
{
    let mut input = String::new();
    loop {
        print!("{}", text);
        io::stdout().flush().unwrap();
        input.clear();
        std::io::stdin().read_line(&mut input).unwrap_or(0);

        if let Ok(val) = input.trim().parse::<T>() {
            if val >= min && val <= max {
                return val;
            }
        };
    }
}

pub fn read_bool_from_terminal(text: String) -> bool {
    let mut input = String::new();
    loop {
        print!("{}", text);
        io::stdout().flush().unwrap();
        input.clear();
        std::io::stdin().read_line(&mut input).unwrap_or(0);

        if input.to_lowercase().contains('y') {
            return true;
        };
        if input.to_lowercase().contains('n') {
            return false;
        };
    }
}

pub fn read_string_from_terminal(text: String) -> String {
    let mut input = String::new();
    print!("{}", text);
    io::stdout().flush().unwrap();
    input.clear();
    std::io::stdin().read_line(&mut input).unwrap_or(0);
    input
}

pub fn read_valid_code_from_terminal(text: String, game: &Game) -> Code {
    loop {
        let solution = read_string_from_terminal(text.clone()).into();
        if !game.is_solution_compatible(&solution) {
            println!(
                "Your solution is invalid ({} digits between 0 and {})",
                game.configuration.column_count,
                game.configuration.base - 1
            );
        } else {
            return solution;
        }
    }
}

async fn server_availability_check() -> Result<bool> {
    print!("Checking server availability... ");
    io::stdout().flush().unwrap();

    let request_url = "http://localhost:3000/ping".to_string();
    let response = reqwest::get(&request_url).await?;

    let s: String = response.json().await?;
    println!("{}", s);
    Ok(true)
}

async fn print_dot_each_second() {
    loop {
        print!(".");
        io::stdout().flush().unwrap();
        sleep(Duration::from_secs(1)).await;
    }
}

async fn get_game_data(base: u8, column_count: u8) -> Result<Game, anyhow::Error> {
    let request_url = format!(
        "http://localhost:3000/generate?base={}&column_count={}",
        base, column_count
    );

    let response = reqwest::get(&request_url).await?;

    response
        .json()
        .await
        .map_err(|reqwest_err| reqwest_err.into())
}

enum Action {
    TestCode,
    ProposeSolution,
    Quit,
}

impl From<u8> for Action {
    fn from(value: u8) -> Self {
        if value == 1 {
            Action::TestCode
        } else if value == 2 {
            Action::ProposeSolution
        } else {
            Action::Quit
        }
    }
}

pub fn display_criterias(game: &Game) {
    for (i, criteria) in game.criterias.iter().enumerate() {
        println!(" {:01}- {}", i, criteria.description);
        for rule in criteria.rules.iter() {
            println!("\t{}", rule);
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("Welcome to...");
    println!(" ______       _       __  __ _           _ ");
    println!("|  ____|     (_)     |  \\/  (_)         | |");
    println!("| |__   _ __  _  __ _| \\  / |_ _ __   __| |");
    println!("|  __| | '_ \\| |/ _` | |\\/| | | '_ \\ / _` |");
    println!("| |____| | | | | (_| | |  | | | | | | (_| |");
    println!("|______|_| |_|_|\\__, |_|  |_|_|_| |_|\\__,_|");
    println!("                 __/ |                     ");
    println!("                |___/                      ");

    server_availability_check().await?;

    let base = read_from_terminal::<u8>("Please choose a base [1-5] : ".to_string(), 1, 5);

    let column_count =
        read_from_terminal::<u8>("Please choose number of columns [1-5] : ".to_string(), 1, 5);

    print!("Waiting for server to generate a secret code");

    let game = select! {
    res =  timeout(Duration::from_secs(10), get_game_data(base, column_count)) => res,
    _ = print_dot_each_second() => unreachable!()}??;

    println!("Done");
    //println!("A game was generated ! Secret code : {}", game.code);

    let mut total_try_count = 0;

    let mut quit = false;

    display_criterias(&game);

    while !quit {
        println!("  1- Test a given code against up to 3 criterias");
        println!("  2- Propose a solution");
        println!("  3- Quit ");

        let main_action: Action =
            read_from_terminal::<u8>("What do you want to do [1-3]: ".to_string(), 1, 5).into();

        match main_action {
            Action::TestCode => {
                let code_test =
                    read_valid_code_from_terminal("Your code to test : ".to_string(), &game);

                let mut try_count = 0;
                let mut retry = true;

                while retry {
                    try_count += 1;
                    total_try_count += 1;

                    let crit_id = read_from_terminal::<u8>(
                        format!(
                            "Which criteria to test with your code [0-{}] : ",
                            game.criterias.len() - 1
                        ),
                        0,
                        (game.criterias.len() - 1) as u8,
                    );

                    let criteria = game.criterias[crit_id as usize].clone();

                    println!(
                        "Result of your code {} against criteria \"{}\" : {}",
                        code_test.clone(),
                        criteria.description,
                        criteria
                            .verif
                            .rule
                            .evaluate(code_test.clone())
                            .unwrap_or(false)
                    );

                    if try_count < 3 {
                        retry = read_bool_from_terminal("Retry [y/n] :".to_string());
                    } else {
                        retry = false;
                    }
                }
            }
            Action::ProposeSolution => {
                let solution = read_valid_code_from_terminal("Your solution : ".to_string(), &game);

                if solution == game.code {
                    println!("Well done ! You have found the right solution !");
                    println!(
                        "The solution was, indeed, {}, found with {} tries",
                        game.code, total_try_count
                    );
                    quit = true;
                } else {
                    println!("Wrong answer !");
                }
            }
            Action::Quit => quit = true,
        };
    }

    Ok(())
}
