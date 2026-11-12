use std::env;

use chrono::{DateTime, Duration, Utc};
use clap::Parser;
use openwearther_response::WeatherResponse;
use serde_json::json;

mod openwearther_response;

const OPENWEATHER_URL: &str = "https://api.openweathermap.org/data/3.0/onecall";
const OPENWEATHER_API_KEY: &str = env!("OPENWEATHER_API_KEY");

#[tokio::main]
async fn main() -> Result<(), ()> {
    let config = Config::parse();
    let forecast = get_todays_forecast(&config).await.unwrap();
    report_checks(forecast, config.uv_threshold);
    Ok(())
}

struct Config {
    lat: f64,
    lng: f64,
    api_key: String,
    uv_threshold: f64,
}

#[derive(Parser)]
struct Args {
    #[arg(long)]
    lat: f64,

    #[arg(long)]
    lng: f64,

    #[arg(long)]
    api_key: Option<String>,

    /// UV index at or above this counts as "high" (WHO: 3+ = use sun protection)
    #[arg(long, default_value_t = 3.0)]
    uv_threshold: f64,
}

impl Config {
    fn parse() -> Self {
        let args = Args::parse();
        let api_key = args.api_key.unwrap_or(String::from(OPENWEATHER_API_KEY));

        Config {
            lat: args.lat,
            lng: args.lng,
            api_key,
            uv_threshold: args.uv_threshold,
        }
    }
}

fn report_checks(forecast: WeatherResponse, uv_threshold: f64) {
    let need_umbrella = need_umbrella(&forecast);
    let details = need_umbrella
        .iter()
        .map(|(_, reason, probability)| format!("{}: {:.0}% regn", reason, probability * 100.))
        .collect::<Vec<_>>()
        .join(", ");
    let rain_severity = need_umbrella
        .iter()
        .map(|(_, _, probability)| *probability)
        .fold(0., f64::max);
    let (uv_max, uv_ranges) = uv_today(&forecast, uv_threshold);
    let response = json!({
        "raincheck": !need_umbrella.is_empty(),
        "details": details,
        "rain_severity": rain_severity,
        "uvcheck": !uv_ranges.is_empty(),
        "uv_max": (uv_max * 10.).round() / 10.,
        "uv_details": uv_ranges.join(", "),
    });
    println!("{}", response);
}

async fn get_todays_forecast(config: &Config) -> Result<WeatherResponse, String> {
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

const FORECAST_WINDOW_SECS: i64 = 24 * 3600;

fn local_time(dt: i64, offset: i32) -> DateTime<Utc> {
    DateTime::from_timestamp(dt + offset as i64, 0).unwrap()
}

fn within_window(dt: i64, now: i64) -> bool {
    dt < now + FORECAST_WINDOW_SECS
}

type UmbrellaCheck = Vec<(bool, String, f64)>;

fn need_umbrella(weather_response: &WeatherResponse) -> UmbrellaCheck {
    let offset = weather_response.timezone_offset;
    let now = weather_response.current.dt;
    let mut results: UmbrellaCheck = Vec::new();

    if weather_response.current.rain.is_some() {
        results.push((true, "Rain now".to_string(), 1.));
    }

    for hour in &weather_response.hourly {
        if !within_window(hour.dt, now) {
            continue;
        }
        if hour.pop > 0.5
            || hour
                .rain
                .as_ref()
                .map_or(false, |h: &openwearther_response::Rain1h| h.hour1 > 0.)
        {
            let time = local_time(hour.dt, offset);
            let plus1h = time + Duration::hours(1);
            let range = format!("{}-{}", time.format("%a %H:%M"), plus1h.format("%H:%M"));
            results.push((true, range, hour.pop));
        }
    }

    results
}

/// Max UV index in the forecast window plus the ranges of hours at or above
/// the threshold, e.g. "Fri 11:00-16:00 (peak UV 5.4)".
fn uv_today(weather_response: &WeatherResponse, threshold: f64) -> (f64, Vec<String>) {
    let offset = weather_response.timezone_offset;
    let now = weather_response.current.dt;
    let mut uv_max = weather_response.current.uvi;
    let mut ranges: Vec<(i64, i64, f64)> = Vec::new(); // start, end, peak

    for hour in &weather_response.hourly {
        if !within_window(hour.dt, now) {
            continue;
        }
        uv_max = uv_max.max(hour.uvi);
        if hour.uvi >= threshold {
            match ranges.last_mut() {
                Some((_, end, peak)) if *end == hour.dt => {
                    *end = hour.dt + 3600;
                    *peak = peak.max(hour.uvi);
                }
                _ => ranges.push((hour.dt, hour.dt + 3600, hour.uvi)),
            }
        }
    }

    let details = ranges
        .iter()
        .map(|(start, end, peak)| {
            format!(
                "{}-{} (peak UV {:.1})",
                local_time(*start, offset).format("%a %H:%M"),
                local_time(*end, offset).format("%H:%M"),
                peak
            )
        })
        .collect();

    (uv_max, details)
}
