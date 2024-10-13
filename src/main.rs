use std::env;

use chrono::{DateTime, Duration};
use clap::Parser;
use openwearther_response::WeatherResponse;
use serde_json::json;

mod openwearther_response;

const OPENWEATHER_URL: &str = "https://api.openweathermap.org/data/3.0/onecall";
const OPENWEATHER_API_KEY: &str = env!("OPENWEATHER_API_KEY");

#[tokio::main]
async fn main() -> Result<(), ()> {
    let config = Config::parse();
    let forecast = get_todays_forecast(config).await.unwrap();
    report_raincheck(forecast);
    Ok(())
}

struct Config {
    lat: f64,
    lng: f64,
    api_key: String,
}

#[derive(Parser)]
struct Args {
    #[arg(long)]
    lat: f64,

    #[arg(long)]
    lng: f64,

    #[arg(long)]
    api_key: Option<String>,
}

impl Config {
    fn parse() -> Self {
        let args = Args::parse();
        let api_key = args.api_key.unwrap_or(String::from(OPENWEATHER_API_KEY));

        Config {
            lat: args.lat,
            lng: args.lng,
            api_key,
        }
    }
}

fn report_raincheck(forecast: WeatherResponse) {
    let need_umbrella = need_umbrella(forecast);
    let details = need_umbrella
        .iter()
        .map(|(_, reason, probability)| format!("{}: {:.1}mm", reason, probability))
        .collect::<Vec<_>>()
        .join(", ");
    let response = json!({
        "raincheck": !need_umbrella.is_empty(),
        "details": details
    });
    println!("{}", response);
}

async fn get_todays_forecast(config: Config) -> Result<WeatherResponse, String> {
    let excluded = ""; // "hourly,daily";
    let addr = format!(
        "{}?lat={}&lon={}&exclude={}&appid={}",
        OPENWEATHER_URL, config.lat, config.lng, excluded, config.api_key
    );
    let body = reqwest::get(addr).await.unwrap().text().await.unwrap();

    match serde_json::from_str(&body) {
        Ok(weather_response) => Ok(weather_response),
        Err(e) => {
            eprint!("An error: {}", e);
            Err(String::from("Error deserialize"))
        }
    }
}

type UmbrellaCheck = Vec<(bool, String, f64)>;

fn need_umbrella(weather_response: WeatherResponse) -> UmbrellaCheck {
    let mut results: UmbrellaCheck = Vec::new();

    if weather_response.current.rain.is_some() {
        results.push((true, "Rain now".to_string(), 1.));
    }

    for hour in weather_response.hourly {
        if hour.pop > 0.5
            || hour
                .rain
                .as_ref()
                .map_or(false, |h: &openwearther_response::Rain1h| h.hour1 > 0.)
        {
            let time = DateTime::from_timestamp(hour.dt, 0).unwrap();
            let plus1h = time + Duration::hours(1);
            let range = format!("{}-{}", time.format("%a %H:%M"), plus1h.format("%H:%M"));
            results.push((true, range, hour.pop));
        }
    }

    results
}
