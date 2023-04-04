use csv::WriterBuilder;
use reqwest::Error;
use serde::Deserialize;
use std::env;
use std::fs::File;

#[derive(Debug, Deserialize)]
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
    let api_key = match env::args().nth(1) {
        Some(key) => key,
        None => {
            eprintln!("Usage: cargo run <api_key>.");
            std::process::exit(1);
        }
    };

    let url = format!(
        "{}/lol/league/v4/challengerleagues/by-queue/RANKED_SOLO_5x5?api_key={}",
        BASE_URL, api_key
    );

    let response = reqwest::get(&url).await?;
    let league: League = response.json().await?;

    let mut top_100_players: Vec<Player> = league.entries.into_iter().collect();

    let file = File::create("korea_leaderboard.csv").expect("Failed to create file.");
    let mut writer = WriterBuilder::new().has_headers(true).from_writer(file);
    writer.write_record(&["Summoner Name", "LP"]).unwrap();

    top_100_players.sort_by_key(|player| std::cmp::Reverse(player.leaguePoints));

    let top_100_players = top_100_players.iter().take(100);

    for player in top_100_players {
        writer
            .write_record(&[&player.summonerName, &player.leaguePoints.to_string()])
            .unwrap();
    }

    Ok(())
}
