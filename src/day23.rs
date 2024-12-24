use core::fmt;

use axum::{
    extract::{Multipart, Path},
    http::StatusCode,
    response::{Html, IntoResponse},
};

pub async fn star() -> impl IntoResponse {
    Html(r#"<div id="star" class="lit"></div>"#)
}

pub async fn present(Path(colour): Path<String>) -> impl IntoResponse {
    let colour = tera::escape_html(&colour);
    let next_colour = match colour.as_str() {
        "red" => "blue",
        "blue" => "purple",
        "purple" => "red",
        _ => return (StatusCode::IM_A_TEAPOT, "Invalid colour".to_string()),
    };
    (
        StatusCode::OK,
        format!(
            r#"
            <div class="present {colour}" hx-get="/23/present/{next_colour}" hx-swap="outerHTML">
                <div class="ribbon"></div>
                <div class="ribbon"></div>
                <div class="ribbon"></div>
                <div class="ribbon"></div>
            </div>
            "#
        ),
    )
}

pub async fn ornament(Path((state, n)): Path<(String, String)>) -> impl IntoResponse {
    let state = tera::escape_html(&state);
    let n = tera::escape_html(&n);

    const TRIGGER_DELAY: &str = "2s";
    let (next_state, current_css_class) = match state.as_str() {
        "on" => ("off", "ornament on"),
        "off" => ("on", "ornament"),
        _ => return (StatusCode::IM_A_TEAPOT, "Invalid state".to_string()),
    };

    (
        StatusCode::OK,
        format!(
            r#"
            <div class="{current_css_class}" id="ornament{n}" hx-trigger="load delay:{TRIGGER_DELAY} once" hx-get="/23/ornament/{next_state}/{n}" hx-swap="outerHTML"></div>
            "#
        ),
    )
}

#[derive(Debug)]
struct ColorHex<'a> {
    red: &'a str,
    green: &'a str,
    blue: &'a str,
}

impl<'a> fmt::Display for ColorHex<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, r#"#{}{}{}"#, self.red, self.green, self.blue)
    }
}

struct LockfileDigest<'a> {
    colour: ColorHex<'a>,
    top: u32,
    left: u32,
}

#[allow(clippy::match_like_matches_macro)]
impl<'a> LockfileDigest<'a> {
    fn from_checksum(checksum: &'a str) -> Option<Self> {
        // Check if the checksum is a valid hexadecimal
        let all_chars_valid = checksum.chars().all(|c| match c {
            '0'..='9' | 'a'..='f' | 'A'..='F' => true,
            _ => false,
        });
        if !all_chars_valid {
            return None;
        }

        let red = checksum.get(0..2)?;
        let green = checksum.get(2..4)?;
        let blue = checksum.get(4..6)?;

        let top = u32::from_str_radix(checksum.get(6..8)?, 16).ok()?;
        let left = u32::from_str_radix(checksum.get(8..10)?, 16).ok()?;

        Some(Self {
            colour: ColorHex { red, green, blue },
            top,
            left,
        })
    }
}

pub async fn lockfile(mut multipart: Multipart) -> Result<String, StatusCode> {
    let mut divs: Vec<String> = vec![];
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?
    {
        if let Some("lockfile") = field.name() {
            let text = field.text().await.map_err(|_| StatusCode::BAD_REQUEST)?;
            let lockfile: toml::map::Map<String, toml::Value> =
                toml::from_str(&text).map_err(|_| StatusCode::BAD_REQUEST)?;

            let dependencies = lockfile
                .get("package")
                .ok_or(StatusCode::BAD_REQUEST)?
                .as_array()
                .ok_or(StatusCode::BAD_REQUEST)?;

            for dep in dependencies {
                let checksum = match dep.get("checksum") {
                    Some(checksum) => checksum.as_str().ok_or(StatusCode::BAD_REQUEST)?,
                    None => {
                        continue;
                    }
                };

                dbg!(&checksum);

                let digest = LockfileDigest::from_checksum(checksum)
                    .ok_or(StatusCode::UNPROCESSABLE_ENTITY)?;

                let div = format!(
                    r#"<div style="background-color:{};top:{}px;left:{}px;"></div>"#,
                    digest.colour, digest.top, digest.left
                );
                divs.push(div);
            }
        }
    }
    dbg!(&divs);
    if divs.is_empty() {
        Err(StatusCode::BAD_REQUEST)
    } else {
        Ok(divs.join("\n"))
    }
}
