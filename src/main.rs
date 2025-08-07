use clap::Parser;
#[derive(Parser)]
#[command(name = "http")]
#[command(about = "A simple HTTP client")]
struct Args {
    url: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    let args = Args::parse();
    let client = reqwest::Client::new();
    println!("Making request to: {}", args.url);
    let response = client.get(&args.url).send().await?;
    println!("Status: {}", response.status());

    let body = response.text().await?;
    println!("Body: {}", body);

    Ok(())
}