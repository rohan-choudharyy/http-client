mod headers;

use clap::{Parser, Subcommand};
use headers::{add_headers_to_request, parse_headers, print_headers, HeaderError};

#[derive(Parser)]
#[command(name = "http")]
#[command(about = "A simple HTTP client")]
struct Args {
    #[command(subcommand)]
    command: HttpMethod,
}

#[derive(Subcommand)]
enum HttpMethod {

    Get{
        url: String,
        #[arg(short = 'H', long = "header", action = clap::ArgAction::Append)]
        headers: Vec<String>,
    },
    Post{
        url: String,
        #[arg(short, long)]
        data: Option<String>,
        #[arg(short = 'H', long = "header", action = clap::ArgAction::Append)]
        headers: Vec<String>,
    },
    Put{
        url: String,
        #[arg(short, long)]
        data: Option<String>,
        #[arg(short = 'H', long = "header", action = clap::ArgAction::Append)]
        headers: Vec<String>,
    },
    Delete{
        url: String,
        #[arg(short = 'H', long = "header", action = clap::ArgAction::Append)]
        headers: Vec<String>,
    },
}

#[derive(Debug)]
enum ClientError {
    Request(reqwest::Error),
    Header(HeaderError),
}

impl std::fmt::Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result{
        match self{
            ClientError::Request(e) => write!(f, "Request error: {}", e),
            ClientError::Header(e) => write!(f, "Header error: {}", e),
        }
    }
}

impl std::error::Error for ClientError {}

impl From<reqwest::Error> for ClientError{
    fn from(error: reqwest::Error) -> Self{
        ClientError::Request(error)
    }
}

impl From<HeaderError> for ClientError{
    fn from(error: HeaderError) -> Self{
        ClientError::Header(error)
    }
}

#[tokio::main]
async fn main() -> Result<(), ClientError>{
    let args = Args::parse();
    let client = reqwest::Client::new();

    match args.command {
        HttpMethod::Get {url, headers} => {
            println!("GET {}", url);

            if !headers.is_empty(){
                if let Ok(header_map) = parse_headers(&headers){
                    print_headers(&header_map, "Request Headers");
                }
            }
            
            let mut request = client.get(&url);
            request = add_headers_to_request(request, &headers)?;
            let response = request.send().await?;
            print_response(response).await?;
        }
        HttpMethod::Post { url, data, headers} => {
            println!("POST {}", url);
            let mut request = client.post(&url);
            request = add_headers_to_request(request, &headers)?;

            if let Some(body) = data { 
                request = request.body(body);
            }
            let response = request.send().await?;
            print_response(response).await?;
        }
        HttpMethod::Put { url, data, headers} => {
            println!("PUT {}", url);
            let mut request = client.put(&url);
            request = add_headers_to_request(request, &headers)?;

            if let Some(body) = data {
                request = request.body(body);
            }

            let response = request.send().await?;
            print_response(response).await?;
        }
        HttpMethod::Delete {url, headers} => {
            println!("DELETE {}", url);
            let mut request = client.delete(&url);
            request = add_headers_to_request(request, &headers)?;
            let response = client.delete(&url).send().await?;
            print_response(response).await?;
        }
    }

    Ok(())
}

async fn print_response(response: reqwest::Response) -> Result<(), ClientError>{
    println!("Status: {}", response.status());

    let important_headers = ["content-type", "content-length", "server"];
    for header_name in &important_headers{
        if let Some(value) = response.headers().get(*header_name){
            println!("{}: {:?}", header_name, value);
        }
    }

    let body = response.text().await?;
    println!("\nBody:");
    println!("Body: {}", body);

    Ok(())
}