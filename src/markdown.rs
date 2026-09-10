use pulldown_cmark::{Event, HeadingLevel, Parser, Tag, TagEnd};

pub fn api_reference_links(markdown: &str) -> Vec<String> {
    let mut links = Vec::new();
    let mut in_api_reference = false;
    let mut in_h2 = false;
    let mut heading = String::new();

    for event in Parser::new(markdown) {
        match event {
            Event::Start(Tag::Heading {
                level: HeadingLevel::H2,
                ..
            }) => {
                in_h2 = true;
                heading.clear();
            }

            Event::Text(text) if in_h2 => {
                heading.push_str(&text);
            }

            Event::End(TagEnd::Heading(HeadingLevel::H2)) => {
                in_h2 = false;

                match heading.trim() {
                    "API Reference" => {
                        in_api_reference = true;
                    }
                    _ if in_api_reference => {
                        in_api_reference = false;
                    }
                    _ => {}
                }
            }

            Event::Start(Tag::Link { dest_url, .. }) if in_api_reference => {
                links.push(dest_url.to_string());
            }

            _ => {}
        }
    }

    links
}