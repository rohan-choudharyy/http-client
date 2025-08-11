use serde_json::Value;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum JsonError {
    InvalidJSon(String),
    ParseError(serde_json::Error),
}

impl fmt::Display for JsonError{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result{
        match self{
            JsonError::InvalidJSon(text) => {
                write!(f, "Invalid JSON: {}", text)
            }
            JsonError::ParseError(e) => {
                write!(f, "JSON parse error: {}", e)
            }
        }
    }
}

impl Error for JsonError {}

impl From<serde_json::Error> for JsonError {
    fn from(error: serde_json::Error) -> Self {
        JsonError::ParseError(error)
    }
}

pub fn is_json_like(text: &str) -> bool {
    let trimmed = text.trim();
    (trimmed.starts_with('{') && trimmed.ends_with('}')) ||
    (trimmed.starts_with('[') && trimmed.ends_with(']'))
}

pub fn pretty_print_json(text: &str) -> Result<String, JsonError>{
    if !is_json_like(text){
        return Err(JsonError::InvalidJSon("Text doesn't look like JSON".to_string()));
    }

    let parsed: Value = serde_json::from_str(text)?;
    let pretty = serde_json::to_string_pretty(&parsed)?;
    Ok(pretty)
}

pub fn pretty_print_json_safe(text: &str) -> String {
    match pretty_print_json(text){
        Ok(pretty) => pretty,
        Err(_) => text.to_string(),
    }
}

pub fn validate_json(text: &str) -> Result<(), JsonError>{
    serde_json::from_str::<Value>(text)?;
    Ok(())
}

pub fn minify_json(text: &str) -> Result<String, JsonError>{
    let parsed: Value = serde_json::from_str(text)?;
    let minified = serde_json::to_string(&parsed)?;
    Ok(minified)
}