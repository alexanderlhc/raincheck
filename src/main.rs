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
        .map(|(_, reason, probability, kind)| {
            format!("{}: {:.0}% {}", reason, probability * 100., kind)
        })
        .collect::<Vec<_>>()
        .join(", ");
    let rain_severity = need_umbrella
        .iter()
        .map(|(_, _, probability, _)| *probability)
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

const DAY_SECS: i64 = 24 * 3600;

fn local_time(dt: i64, offset: i32) -> DateTime<Utc> {
    DateTime::from_timestamp(dt + offset as i64, 0).unwrap()
}

/// The rest of today: hours that start before the coming local midnight.
fn within_window(dt: i64, now: i64, offset: i32) -> bool {
    let offset = offset as i64;
    let midnight = ((now + offset).div_euclid(DAY_SECS) + 1) * DAY_SECS - offset;
    dt < midnight
}

type UmbrellaCheck = Vec<(bool, String, f64, &'static str)>;

fn falls(p: &Option<openwearther_response::Rain1h>) -> bool {
    p.as_ref().is_some_and(|h| h.hour1 > 0.)
}

/// "regn", "sne" or "regn/sne": what falls. Snow alone only when snow is forecast and rain is not.
fn precipitation_kind(rain: bool, snow: bool) -> &'static str {
    match (rain, snow) {
        (false, true) => "sne",
        (true, true) => "regn/sne",
        _ => "regn",
    }
}

fn need_umbrella(weather_response: &WeatherResponse) -> UmbrellaCheck {
    let offset = weather_response.timezone_offset;
    let now = weather_response.current.dt;
    let mut results: UmbrellaCheck = Vec::new();

    let current = &weather_response.current;
    if current.rain.is_some() || current.snow.is_some() {
        let kind = precipitation_kind(current.rain.is_some(), current.snow.is_some());
        let reason = match kind {
            "sne" => "Snow now",
            "regn/sne" => "Rain/snow now",
            _ => "Rain now",
        };
        results.push((true, reason.to_string(), 1., kind));
    }

    for hour in &weather_response.hourly {
        if !within_window(hour.dt, now, offset) {
            continue;
        }
        let rain = falls(&hour.rain);
        let snow = falls(&hour.snow);
        if hour.pop > 0.5 || rain || snow {
            // Weather condition 6xx is snow (https://openweathermap.org/weather-conditions):
            // it names what a likely-but-unmeasured hour will bring.
            let snow = snow || hour.weather.iter().any(|w| (600..700).contains(&w.id));
            let time = local_time(hour.dt, offset);
            let plus1h = time + Duration::hours(1);
            let range = format!("{}-{}", time.format("%a %H:%M"), plus1h.format("%H:%M"));
            results.push((true, range, hour.pop, precipitation_kind(rain, snow)));
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
        if !within_window(hour.dt, now, offset) {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> serde_json::Value {
        serde_json::from_str(include_str!("../sample-response.json")).unwrap()
    }

    fn dry(v: &mut serde_json::Value) {
        let current = v["current"].as_object_mut().unwrap();
        current.remove("rain");
        current.remove("snow");
        for h in v["hourly"].as_array_mut().unwrap() {
            let h = h.as_object_mut().unwrap();
            h.remove("rain");
            h.remove("snow");
            h.insert("pop".into(), serde_json::json!(0.0));
            h.insert("weather".into(), serde_json::json!([{"id": 800, "main": "Clear", "description": "clear sky", "icon": "01d"}]));
        }
    }

    fn check(v: serde_json::Value) -> UmbrellaCheck {
        need_umbrella(&serde_json::from_value(v).unwrap())
    }

    #[test]
    fn dry_forecast_needs_no_umbrella() {
        let mut v = sample();
        dry(&mut v);
        assert!(check(v).is_empty());
    }

    #[test]
    fn snow_in_an_hour_counts() {
        let mut v = sample();
        dry(&mut v);
        v["hourly"][1]["snow"] = serde_json::json!({"1h": 0.3});
        let r = check(v);
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].3, "sne");
    }

    #[test]
    fn snow_condition_without_amount_counts_when_likely() {
        let mut v = sample();
        dry(&mut v);
        v["hourly"][1]["pop"] = serde_json::json!(0.8);
        v["hourly"][1]["weather"][0]["id"] = serde_json::json!(601);
        let r = check(v);
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].3, "sne");
    }

    #[test]
    fn snow_now_counts() {
        let mut v = sample();
        dry(&mut v);
        v["current"]["snow"] = serde_json::json!({"1h": 0.5});
        let r = check(v);
        assert_eq!(r[0].1, "Snow now");
        assert_eq!(r[0].3, "sne");
    }

    #[test]
    fn snow_condition_alone_is_not_enough() {
        let mut v = sample();
        dry(&mut v);
        v["hourly"][1]["pop"] = serde_json::json!(0.3);
        v["hourly"][1]["weather"][0]["id"] = serde_json::json!(600);
        assert!(check(v).is_empty());
    }

    #[test]
    fn rain_and_snow_together() {
        let mut v = sample();
        dry(&mut v);
        v["hourly"][1]["rain"] = serde_json::json!({"1h": 0.2});
        v["hourly"][1]["snow"] = serde_json::json!({"1h": 0.1});
        assert_eq!(check(v)[0].3, "regn/sne");
    }

    // The sample's "now" is 22:06 local (UTC+2): hourly[1] is 22:00 today,
    // hourly[2] is 00:00 tomorrow.
    #[test]
    fn rain_later_today_counts() {
        let mut v = sample();
        dry(&mut v);
        v["hourly"][1]["rain"] = serde_json::json!({"1h": 0.4});
        assert_eq!(check(v).len(), 1);
    }

    #[test]
    fn rain_after_midnight_is_ignored() {
        let mut v = sample();
        dry(&mut v);
        v["hourly"][2]["rain"] = serde_json::json!({"1h": 0.4});
        v["hourly"][5]["pop"] = serde_json::json!(0.9);
        assert!(check(v).is_empty());
    }

    #[test]
    fn uv_after_midnight_is_ignored() {
        let mut v = sample();
        dry(&mut v);
        v["current"]["uvi"] = serde_json::json!(0.0);
        for h in v["hourly"].as_array_mut().unwrap() {
            h["uvi"] = serde_json::json!(0.0);
        }
        v["hourly"][14]["uvi"] = serde_json::json!(6.0);
        let (max, ranges) = uv_today(&serde_json::from_value(v).unwrap(), 3.0);
        assert_eq!(max, 0.0);
        assert!(ranges.is_empty());
    }

    #[test]
    fn window_ends_at_local_midnight() {
        // 2025-11-13 06:00 CET (05:00 UTC), offset +1h
        let now = 1_763_010_000;
        let offset = 3600;
        let midnight_utc = 1_763_074_800; // 2025-11-13 23:00 UTC
        assert!(within_window(midnight_utc - 3600, now, offset));
        assert!(!within_window(midnight_utc, now, offset));
    }
}
