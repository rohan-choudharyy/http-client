use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum HeaderError {
    InvalidFormat(String),
    InvalidName(String),
    InvalidValue(String),
}

impl fmt::Display for HeaderError{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result{
        match self{
            HeaderError::InvalidFormat(header) => {
                write!(f, "Invalid header format: '{}'. Use 'Key: Value' format", header)
            }
            HeaderError::InvalidName(name) => {
                write!(f, "Invalid header name: '{}'", name)
            }
            HeaderError::InvalidValue(value) => {
                write!(f, "Invalid header value: '{}'", value)
            }
        }
    }
}

impl Error for HeaderError {}

pub fn parse_headers(headers: &[String]) -> Result<HeaderMap, HeaderError> {
    let mut header_map = HeaderMap::new();

    for header in headers {
        if let Some((key, value)) = header.split_once(':'){
            let key = key.trim();
            let value = value.trim();

            let header_name: HeaderName = key.parse()
            .map_err(|_| HeaderError::InvalidName(key.to_string()))?;

            let header_value: HeaderValue = value.parse()
            .map_err(|_| HeaderError::InvalidValue(value.to_string()))?;

            header_map.insert(header_name, header_value);
        } else {
            return Err(HeaderError::InvalidFormat(header.clone()));
        }
    }
    Ok(header_map)
}

pub fn add_headers_to_request(
    request: reqwest::RequestBuilder,
    headers: &[String],
) -> Result<reqwest::RequestBuilder, HeaderError>{
    if headers.is_empty(){
        return Ok(request);
    }
    let header_map = parse_headers(headers)?;
    Ok(request.headers(header_map))
}

pub fn print_headers(headers: &HeaderMap, title: &str){
    if headers.is_empty(){
        return;
    }

    println!("{}", title);
    for(name, value) in headers {
        println!(" {}: {:?}", name, value);
    }
}