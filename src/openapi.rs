use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Parser, Tag, TagEnd};
use utoipa::openapi::{Components, OpenApi};

pub fn merge_openapi_schemas(schemas: Vec<OpenApi>) -> Option<OpenApi> {
    let mut schemas = schemas.into_iter();

    let mut merged = schemas.next()?;

    for schema in schemas {
        merged = merge_schema(merged, schema);
    }

    let mut json = serde_json::to_value(&merged).expect("OpenApi should be serializable");

    cleanup_openapi_json(&mut json);

    Some(serde_json::from_value(json).expect("cleaned OpenApi JSON should deserialize"))
}

fn cleanup_openapi_json(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(map) => {
            map.retain(|key, _| !key.to_ascii_lowercase().starts_with("x-readme"));

            for (key, value) in map.iter_mut() {
                if key == "value"
                    && let serde_json::Value::String(string) = value
                    && let Ok(parsed) = serde_json::from_str::<serde_json::Value>(string)
                {
                    *value = parsed;
                    continue;
                }

                cleanup_openapi_json(value);
            }
        }

        serde_json::Value::Array(array) => {
            for value in array {
                cleanup_openapi_json(value);
            }
        }

        _ => {}
    }
}

fn merge_schema(mut base: OpenApi, other: OpenApi) -> OpenApi {
    // Merge paths.
    for (path, item) in other.paths.paths {
        base.paths.paths.insert(path, item);
    }

    match (&mut base.components, other.components) {
        (Some(base_components), Some(other_components)) => {
            merge_components(base_components, other_components);
        }
        (None, Some(other_components)) => {
            base.components = Some(other_components);
        }
        _ => {}
    }

    match (&mut base.servers, other.servers) {
        (Some(base_servers), Some(other_servers)) => {
            for server in other_servers {
                if !base_servers.iter().any(|s| s.url == server.url) {
                    base_servers.push(server);
                }
            }
        }
        (None, Some(other_servers)) => {
            base.servers = Some(other_servers);
        }
        _ => {}
    }

    match (&mut base.tags, other.tags) {
        (Some(base_tags), Some(other_tags)) => {
            for tag in other_tags {
                if !base_tags.iter().any(|t| t.name == tag.name) {
                    base_tags.push(tag);
                }
            }
        }
        (None, Some(other_tags)) => {
            base.tags = Some(other_tags);
        }
        _ => {}
    }

    match (&mut base.security, other.security) {
        (Some(base_security), Some(other_security)) => {
            for security in other_security {
                if !base_security.contains(&security) {
                    base_security.push(security);
                }
            }
        }
        (None, Some(other_security)) => {
            base.security = Some(other_security);
        }
        _ => {}
    }

    base
}

fn merge_components(base: &mut Components, other: Components) {
    base.schemas.extend(other.schemas);
    base.responses.extend(other.responses);
    base.security_schemes.extend(other.security_schemes);
}

pub fn try_get_openapi_json(markdown: &str) -> Result<Option<OpenApi>, serde_json::Error> {
    let parser = Parser::new(markdown);

    let mut in_openapi_section = false;
    let mut in_code_block = false;
    let mut heading_text = String::new();
    let mut openapi_json = String::new();

    for event in parser {
        match event {
            Event::Start(Tag::Heading {
                level: HeadingLevel::H1,
                ..
            }) => {
                heading_text.clear();
            }

            Event::Text(text) if !in_code_block => {
                heading_text.push_str(&text);
            }

            Event::End(TagEnd::Heading(HeadingLevel::H1)) => {
                in_openapi_section =
                    heading_text.trim().to_lowercase() == "OpenAPI definition".to_lowercase();
            }

            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(language))) if in_openapi_section => {
                if language.is_empty() || language.as_ref() == "json" {
                    in_code_block = true;
                    openapi_json.clear();
                }
            }

            Event::Text(text) if in_code_block => {
                openapi_json.push_str(&text);
            }

            Event::End(TagEnd::CodeBlock) if in_code_block => {
                return serde_json::from_str::<OpenApi>(&openapi_json).map(Some);
            }

            _ => {}
        }
    }

    Ok(None)
}
