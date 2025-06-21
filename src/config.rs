use once_cell::sync::Lazy;
use serde::Deserialize;
use std::fs;
use url::Url;

#[derive(Debug, Deserialize, Clone)]
pub struct Variables {
    pub ws_url: Url,
    pub http_url: Url,
}

pub type ParseError = String;

pub static VARIABLES: Lazy<Variables> = Lazy::new(|| {
    let config_str = fs::read_to_string("config.toml").unwrap();

    let variables: Variables = toml::from_str(&config_str).unwrap();

    variables
});

pub fn get_variables() -> &'static Variables {
    &VARIABLES
}
