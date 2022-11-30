use std::time::Duration;

use anyhow::Result;
use enigmind_lib::setup::Game;

use tokio::time::timeout;

pub async fn get_game(base: u8, column_count: u8) -> Result<Game> {
    let request_url = format!(
        "http://localhost:3000/generate?base={}&column_count={}",
        base, column_count
    );
    let response = reqwest::get(&request_url).await?;

    let g: Game = response.json().await?;
    Ok(g)
}

pub fn is_base_valid(base: u8) -> bool {
    base > 0 && base <= 6
}

pub fn read_u8_from_terminal(text: String, min: u8, max: u8) -> u8 {
    let mut input = String::new();

    let mut res = 0;
    while res < min || res > max {
        println!("{}", text);

        input.clear();
        std::io::stdin().read_line(&mut input).unwrap_or(0);
        //println!("{} {}", l, input);
        res = input.trim().parse::<u8>().unwrap_or(0);
    }

    res
}

async fn server_availability_check() -> Result<bool> {
    let request_url = "http://localhost:3000/handshake".to_string();
    let response = reqwest::get(&request_url).await?;

    let s: String = response.json().await?;
    println!("{}", s);
    Ok(true)
}

/*async fn infinite_print_dot() {
    loop {
        print!(".");
        sleep(Duration::from_secs(1)).await;
    }
}*/

#[tokio::main]
async fn main() -> Result<()> {
    println!("Welcome to...");
    println!(" ______       _       __  __ _           _  ");
    println!("|  ____|     (_)     |  \\/  (_)         | |");
    println!("| |__   _ __  _  __ _| \\  / |_ _ __   __| |");
    println!("|  __| | '_ \\| |/ _` | |\\/| | | '_ \\ / _` |");
    println!("| |____| | | | | (_| | |  | | | | | | (_| |");
    println!("|______|_| |_|_|\\__, |_|  |_|_|_| |_|\\__,_|");
    println!("                 __/ |                     ");
    println!("                |___/                      ");

    print!("Checking server availability... ");
    server_availability_check().await?;

    let base = read_u8_from_terminal("Please choose a base [1-5] :".to_string(), 1, 5);

    let column_count =
        read_u8_from_terminal("Please choose number of columns [1-5] :".to_string(), 1, 5);

    print!("Waiting for server to generate a secret code... ");

    //let game = timeout(Duration::from_secs(10), get_game(base, column_count)).await?.map_err(|_| "No response from server (server took too long)".to_string())?;

    /*let res = select! {
    r1 = timeout(Duration::from_secs(10), get_game(base, column_count)) => r1,
    r2 = infinite_print_dot() => r2};*/

    match timeout(Duration::from_secs(10), get_game(base, column_count)).await? {
        Ok(game) => println!("{}", game),
        Err(_) => println!("No response from server (server took too long"),
    };

    Ok(())
}
