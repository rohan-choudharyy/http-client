use clap::{Parser, Subcommand};
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
    },
    Post{
        url: String,
        #[arg(short, long)]
        data: Option<String>,
    },
    Put{
        url: String,
        #[arg(short, long)]
        data: Option<String>,
    },
    Delete{
        url: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    let args = Args::parse();
    let client = reqwest::Client::new();

    match args.command {
        HttpMethod::Get {url} => {
            let response = client.get(&url).send().await?;
            print_response(response).await?;
        }
        HttpMethod::Post { url, data } => {
            println!("POST {}", url);
            let mut request = client.post(&url);

            if let Some(body) = data { 
                request = request.body(body);
            }
            let response = request.send().await?;
            print_response(response).await?;
        }
        HttpMethod::Put { url, data } => {
            println!("PUT {}", url);
            let mut request = client.put(&url);

            if let Some(body) = data {
                request = request.body(body);
            }
            let response = request.send().await?;
            print_response(response).await?;
        }
        HttpMethod::Delete {url} => {
            println!("DELETE {}", url);
            let response = client.delete(&url).send().await?;
            print_response(response).await?;
        }
    }

    Ok(())
}

async fn print_response(response: reqwest::Response) -> Result<(), Box<dyn std::error::Error>>{
    println!("Status: {}", response.status());

    if let Some(content_type) = response.headers().get("content-type"){
        println!("Content-Tyoe: {:?}", content_type);
    }
    let body = response.text().await?;
    println!("Body: {}", body);

    Ok(())
}