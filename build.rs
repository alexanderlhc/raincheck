use dotenv::dotenv;
use std::env;

fn main() {
    dotenv().ok(); // Ignores errors if .env is not found

    let api_key = env::var("OPENWEATHER_API_KEY")
        .expect("OPENWEATHER_API_KEY environment variable not set in .env file");

    println!("cargo:rustc-env=OPENWEATHER_API_KEY={}", api_key);
}

