use csv::WriterBuilder;
use reqwest::{Client, Error};
use scraper::{Html, Selector};
use serde::Deserialize;
use std::env;
use std::fs::File;

#[derive(Debug, Deserialize, Clone)] // Add `Clone` to the derive list
#[allow(non_snake_case, dead_code)]
struct Player {
    summonerName: String,
    leaguePoints: i32,
}

#[derive(Deserialize)]
struct League {
    entries: Vec<Player>,
}

const BASE_URL: &str = "https://kr.api.riotgames.com";

#[tokio::main]
async fn main() -> Result<(), Error> {
    let client = Client::new();

    let api_key = match env::args().nth(1) {
        Some(key) => key,
        None => {
            eprintln!("Usage: cargo run <api_key> <number_of_players_to_scout.");
            std::process::exit(1);
        }
    };

    let n_players = match env::args().nth(2) {
        Some(n) => n,
        None => {
            eprintln!("Usage: cargo run <api_key> <number_of_players_to_scout.");
            std::process::exit(2);
        }
    };

    let n_players: usize = n_players.trim().parse().expect("provide a smaller number.");

    let url = format!(
        "{}/lol/league/v4/challengerleagues/by-queue/RANKED_SOLO_5x5?api_key={}",
        BASE_URL, api_key
    );

    let response = client.get(&url).send().await?;
    let league: League = response.json().await?;

    let mut top_100_players: Vec<Player> = league.entries.into_iter().collect();

    let file = File::create("korea_leaderboard.csv").expect("failed to create file.");
    let mut writer = WriterBuilder::new().has_headers(false).from_writer(file);

    top_100_players.sort_by_key(|player| std::cmp::Reverse(player.leaguePoints));

    let top_100_players = top_100_players.iter().take(100);

    for player in top_100_players.clone().take(n_players) {
        let url = format!("https://www.op.gg/summoners/kr/{}", player.summonerName);
        let response = client.get(&url).send().await?;
        let body = response.text().await?;

        let document = Html::parse_document(&body);
        let champion_selector = Selector::parse(".champion-box").unwrap();
        let champion_list = document.select(&champion_selector);

        for champion_element in champion_list.take(3) {
            let name_selector = Selector::parse(".name a").unwrap();
            let played_selector = Selector::parse(".played .count").unwrap();

            let champion_name = champion_element
                .select(&name_selector)
                .next()
                .unwrap()
                .text()
                .collect::<String>();
            let played = champion_element
                .select(&played_selector)
                .next()
                .unwrap()
                .text()
                .collect::<String>();

            writer
                .write_record(&[champion_name, played])
                .expect("failed to write to file.");
        }
        writer
            .write_record(&["---", "---"])
            .expect("failed to write to file.")
    }

    Ok(())
}
