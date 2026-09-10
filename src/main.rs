#![forbid(unsafe_code)]

pub mod constants;
pub mod macros;
pub mod markdown;
pub mod openapi;

use std::fs;

use env_logger::{DEFAULT_FILTER_ENV, Env};
use log::info;

use crate::{constants::API_REFERENCE_DOCUMENT_URL, markdown::api_reference_links, openapi::{merge_openapi_schemas, try_get_openapi_json}};

pub fn fetch_string_from_url(url: &str) -> Result<String, reqwest::Error> {
    let request_client = reqwest::blocking::Client::new();

    let request = request_client
        .get(url)
        .header("User-Agent", user_agent_header!());

    request.send().expect("Couldnt fetch").text()
}

fn main() {
    let env = Env::default().filter_or(DEFAULT_FILTER_ENV, "info");

    env_logger::init_from_env(env);

    info!("Starting schema generation");

    let markdown = fetch_string_from_url(API_REFERENCE_DOCUMENT_URL).expect("Failed to fetch base");

    let api_reference_markdown_links = api_reference_links(markdown.as_ref());

    let link_amount = api_reference_markdown_links.len();

    info!("Found {link_amount} links");

let maybe_openapi_schemas = api_reference_markdown_links
    .iter()
    .filter_map(|url| fetch_string_from_url(url).ok())
    .filter_map(|markdown| try_get_openapi_json(&markdown).ok())
    .flatten();

    let api = merge_openapi_schemas(maybe_openapi_schemas.collect()).unwrap();

    fs::write("schema.json", api.to_pretty_json().unwrap()).unwrap();
}
