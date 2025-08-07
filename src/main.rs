#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    let url = "https://httpbin.org/get";
    let client = reqwest::Client::new();
    let response = client.get(url).send().await?;
    println!("Status: {}", response.status());

    let body = response.text().await?;
    println!("Body: {}", body);

    Ok(())
}