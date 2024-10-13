use serde::Deserialize;

// openweather api response = dead code
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct WeatherResponse {
    pub lat: f64,
    pub lon: f64,
    pub timezone: String,
    pub timezone_offset: i32,
    pub current: CurrentWeather,
    pub minutely: Vec<MinutelyWeather>,
    pub hourly: Vec<HourlyWeather>,
    pub daily: Vec<DailyWeather>,
    pub alerts: Option<Vec<WeatherAlert>>,
}

// openweather api response = dead code
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct CurrentWeather {
    pub dt: i64,
    pub sunrise: Option<i64>,
    pub sunset: Option<i64>,
    pub temp: f64,
    pub feels_like: f64,
    pub pressure: i32,
    pub humidity: i32,
    pub dew_point: f64,
    pub uvi: f64,
    pub clouds: i32,
    pub visibility: i32,
    pub wind_speed: f64,
    pub wind_gust: Option<f64>,
    pub wind_deg: i32,
    pub rain: Option<Rain1h>, // Precipitation in mm/h
    pub snow: Option<f64>,    // Precipitation in mm/h
    pub weather: Vec<WeatherCondition>,
}

// openweather api response = dead code
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct WeatherCondition {
    pub id: i32,
    pub main: String,
    pub description: String,
    pub icon: String,
}

// openweather api response = dead code
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct MinutelyWeather {
    pub dt: i64,
    pub precipitation: f64, // Precipitation in mm/h
}

// openweather api response = dead code
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct HourlyWeather {
    pub dt: i64,
    pub temp: f64,
    pub feels_like: f64,
    pub pressure: i32,
    pub humidity: i32,
    pub dew_point: f64,
    pub uvi: f64,
    pub clouds: i32,
    pub visibility: i32,
    pub wind_speed: f64,
    pub wind_gust: Option<f64>,
    pub wind_deg: i32,
    pub pop: f64,             // Probability of precipitation (0 to 1)
    pub rain: Option<Rain1h>, // Precipitation in mm/h
    pub snow: Option<f64>,    // Precipitation in mm/h
    pub weather: Vec<WeatherCondition>,
}

// openweather api response = dead code
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct DailyWeather {
    pub dt: i64,
    pub sunrise: Option<i64>,
    pub sunset: Option<i64>,
    pub moonrise: i64,
    pub moonset: i64,
    pub moon_phase: f64,
    pub summary: String,
    pub temp: DailyTemperature,
    pub feels_like: DailyFeelsLike,
    pub pressure: i32,
    pub humidity: i32,
    pub dew_point: f64,
    pub wind_speed: f64,
    pub wind_gust: Option<f64>,
    pub wind_deg: i32,
    pub clouds: i32,
    pub pop: f64,          // Probability of precipitation (0 to 1)
    pub rain: Option<f32>, // Precipitation volume in mm
    pub snow: Option<f64>, // Snow volume in mm
    pub weather: Vec<WeatherCondition>,
}

// openweather api response = dead code
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct DailyTemperature {
    pub morn: f64,
    pub day: f64,
    pub eve: f64,
    pub night: f64,
    pub min: f64,
    pub max: f64,
}

// openweather api response = dead code
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct DailyFeelsLike {
    pub morn: f64,
    pub day: f64,
    pub eve: f64,
    pub night: f64,
}

// openweather api response = dead code
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct WeatherAlert {
    pub sender_name: String,
    pub event: String,
    pub start: i64,
    pub end: i64,
    pub description: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct Rain1h {
    #[serde(rename = "1h")]
    pub hour1: f32,
}
