mod headers;
mod json;
mod tui;

use clap::{Parser, Subcommand};
use headers::{add_headers_to_request, parse_headers, print_headers, HeaderError};
use json::{pretty_print_json_safe, JsonError};

#[derive(Parser)]
#[command(name = "http")]
#[command(about = "A simple HTTP client")]
struct Args {
    #[command(subcommand)]
    command: HttpMethod,
}

#[derive(Subcommand)]
enum HttpMethod {
    Get {
        url: String,
        #[arg(short = 'H', long = "header", action = clap::ArgAction::Append)]
        headers: Vec<String>,
    },
    Post {
        url: String,
        #[arg(short, long)]
        data: Option<String>,
        #[arg(short, long)]
        json: Option<String>,
        #[arg(short = 'H', long = "header", action = clap::ArgAction::Append)]
        headers: Vec<String>,
    },
    Put {
        url: String,
        #[arg(short, long)]
        data: Option<String>,
        #[arg(short, long)]
        json: Option<String>,
        #[arg(short = 'H', long = "header", action = clap::ArgAction::Append)]
        headers: Vec<String>,
    },
    Delete {
        url: String,
        #[arg(short = 'H', long = "header", action = clap::ArgAction::Append)]
        headers: Vec<String>,
    },
    Tui,
}

#[derive(Debug)]
enum ClientError {
    Request(reqwest::Error),
    Header(HeaderError),
    Json(JsonError),
    Tui(Box<dyn std::error::Error>), // Add a new variant for TUI errors
}

impl std::fmt::Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClientError::Request(e) => write!(f, "Request error: {}", e),
            ClientError::Header(e) => write!(f, "Header error: {}", e),
            ClientError::Json(e) => write!(f, "JSON error: {}", e),
            ClientError::Tui(e) => write!(f, "TUI error: {}", e),
        }
    }
}

impl std::error::Error for ClientError {}

impl From<reqwest::Error> for ClientError {
    fn from(error: reqwest::Error) -> Self {
        ClientError::Request(error)
    }
}

impl From<JsonError> for ClientError {
    fn from(error: JsonError) -> Self {
        ClientError::Json(error)
    }
}

impl From<HeaderError> for ClientError {
    fn from(error: HeaderError) -> Self {
        ClientError::Header(error)
    }
}

#[tokio::main]
async fn main() -> Result<(), ClientError> {
    let args = Args::parse();
    let client = reqwest::Client::new();

    match args.command {
        HttpMethod::Get { url, headers } => {
            println!("GET {}", url);

            if !headers.is_empty() {
                if let Ok(header_map) = parse_headers(&headers) {
                    print_headers(&header_map, "Request Headers");
                }
            }
            
            let mut request = client.get(&url);
            request = add_headers_to_request(request, &headers)?;
            let response = request.send().await?;
            print_response(response).await?;
        }
        HttpMethod::Post { url, data, json, headers } => {
            println!("POST {}", url);
            
            match (data.as_ref(), json.as_ref()) {
                (Some(_), Some(_)) => {
                    return Err(ClientError::Json(JsonError::InvalidJSon(
                        "Cannot use both --data and --json options".to_string()
                    )));
                }
                _ => {}
            }

            if !headers.is_empty() {
                if let Ok(header_map) = parse_headers(&headers) {
                    print_headers(&header_map, "Request Headers");
                }
            }

            let mut request = client.post(&url);
            request = add_headers_to_request(request, &headers)?;

            if let Some(json_data) = json {
                json::validate_json(&json_data)?;
                request = request
                    .header("Content-Type", "application/json")
                    .body(json_data);
                println!("Sending JSON data");
            } else if let Some(raw_data) = data {
                request = request.body(raw_data);
                println!("Sending raw data");
            }
            let response = request.send().await?;
            print_response(response).await?;
        }

        HttpMethod::Put { url, data, json, headers } => {
            println!("PUT {}", url);
            
            match (data.as_ref(), json.as_ref()) {
                (Some(_), Some(_)) => {
                    return Err(ClientError::Json(JsonError::InvalidJSon(
                        "Cannot use both --data and --json options".to_string()
                    )));
                }
                _ => {}
            }

            if !headers.is_empty() {
                if let Ok(header_map) = parse_headers(&headers) {
                    print_headers(&header_map, "Request Headers");
                }
            }

            let mut request = client.put(&url);
            request = add_headers_to_request(request, &headers)?;

            if let Some(json_data) = json {
                json::validate_json(&json_data)?;
                request = request
                    .header("Content-Type", "application/json")
                    .body(json_data);
                println!("Sending JSON data");
            } else if let Some(raw_data) = data {
                request = request.body(raw_data);
                println!("Sending raw data");
            }
            let response = request.send().await?;
            print_response(response).await?;
        }
        HttpMethod::Delete { url, headers } => {
            println!("DELETE {}", url);
            let mut request = client.delete(&url);
            request = add_headers_to_request(request, &headers)?;
            let response = request.send().await?;
            print_response(response).await?;
        }
        HttpMethod::Tui => {
            println!("Launching TUI mode...");
            if let Err(e) = tui::run_tui().await {
                eprintln!("TUI error: {}", e);
                return Err(ClientError::Tui(e));
            }
        }
    }

    Ok(())
}

async fn print_response(response: reqwest::Response) -> Result<(), ClientError> {
    println!("Status: {}", response.status());

    let important_headers = ["content-type", "content-length", "server"];
    let headers = response.headers();
    for header_name in &important_headers {
        if let Some(value) = headers.get(*header_name) {
            println!("{}: {:?}", header_name, value);
        }
    }

    // Extract content-type before consuming response
    let content_type = headers
        .get("content-type")
        .and_then(|ct| ct.to_str().ok())
        .unwrap_or("")
        .to_string(); // Convert to owned String

    let body = response.text().await?;

    println!("\nResponse Body:");
    if content_type.contains("application/json") || json::is_json_like(&body) {
        let pretty_json = pretty_print_json_safe(&body);
        println!("{}", pretty_json);
    } else {
        println!("{}", body);
    }

    Ok(())
}