//! Weather Plugin Example
//!
//! Demonstrates how to use network capabilities to fetch weather data from an external API.

use extism_pdk::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Weather request
#[derive(Deserialize)]
struct WeatherRequest {
    city: String,
    country: Option<String>,
}

/// Weather data
#[derive(Serialize)]
struct WeatherData {
    city: String,
    temperature: f32,
    description: String,
    humidity: u32,
}

/// Weather response
#[derive(Serialize)]
struct WeatherResponse {
    success: bool,
    data: Option<WeatherData>,
    error: Option<String>,
}

/// Fetch weather data for a city
/// This demonstrates how a plugin would make HTTP requests to external APIs
#[plugin_fn]
pub fn get_weather(input: String) -> FnResult<String> {
    // Parse input
    let request: WeatherRequest = match serde_json::from_str(&input) {
        Ok(req) => req,
        Err(e) => {
            let response = WeatherResponse {
                success: false,
                data: None,
                error: Some(format!("Invalid request: {}", e)),
            };
            return Ok(serde_json::to_string(&response)?);
        }
    };

    // Log the request
    log(
        LogLevel::Info,
        &format!("Fetching weather for: {}", request.city),
    )?;

    // In a real implementation, this would make an actual HTTP request
    // For now, we'll simulate the response
    let weather = simulate_weather_fetch(&request.city);

    let response = WeatherResponse {
        success: true,
        data: Some(weather),
        error: None,
    };

    Ok(serde_json::to_string(&response)?)
}

/// Simulate fetching weather data
/// In a real implementation, this would call an external weather API
fn simulate_weather_fetch(city: &str) -> WeatherData {
    // Simulate different weather for different cities
    let (temp, description, humidity) = match city.to_lowercase().as_str() {
        "london" => (15.5, "Cloudy", 75),
        "paris" => (18.2, "Sunny", 60),
        "tokyo" => (22.8, "Partly Cloudy", 70),
        "new york" => (20.1, "Rainy", 80),
        "sydney" => (25.3, "Clear", 55),
        _ => (20.0, "Unknown", 65),
    };

    WeatherData {
        city: city.to_string(),
        temperature: temp,
        description: description.to_string(),
        humidity,
    }
}

/// Fetch weather for multiple cities
#[plugin_fn]
pub fn get_batch_weather(input: String) -> FnResult<String> {
    #[derive(Deserialize)]
    struct BatchRequest {
        cities: Vec<String>,
    }

    #[derive(Serialize)]
    struct BatchResponse {
        success: bool,
        results: Vec<WeatherData>,
        error: Option<String>,
    }

    // Parse input
    let request: BatchRequest = match serde_json::from_str(&input) {
        Ok(req) => req,
        Err(e) => {
            let response = BatchResponse {
                success: false,
                results: vec![],
                error: Some(format!("Invalid request: {}", e)),
            };
            return Ok(serde_json::to_string(&response)?);
        }
    };

    log(
        LogLevel::Info,
        &format!("Fetching weather for {} cities", request.cities.len()),
    )?;

    // Fetch weather for each city
    let results: Vec<WeatherData> = request
        .cities
        .iter()
        .map(|city| simulate_weather_fetch(city))
        .collect();

    let response = BatchResponse {
        success: true,
        results,
        error: None,
    };

    Ok(serde_json::to_string(&response)?)
}

/// Plugin metadata
#[plugin_fn]
pub fn metadata() -> FnResult<String> {
    let metadata = serde_json::json!({
        "name": "weather-plugin",
        "version": "0.1.0",
        "description": "Fetches weather data from external APIs",
        "author": "AgentMem Team",
        "plugin_type": "DataSource",
        "required_capabilities": ["NetworkAccess", "LoggingAccess"]
    });

    Ok(metadata.to_string())
}

