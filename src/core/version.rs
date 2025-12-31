use colored::Colorize;
use native_tls::TlsConnector;
use serde::Deserialize;
use std::error::Error;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;

/// Repo details
const REPO_OWNER: &str = "kodelint";
const REPO_NAME: &str = "git-selective-ignore";

/// Get local version from Cargo.toml at compile time
fn get_local_version() -> Result<String, Box<dyn Error>> {
    Ok(env!("CARGO_PKG_VERSION").to_string())
}

/// GitHub release response
#[derive(Deserialize)]
struct GitHubRelease {
    tag_name: String,
}

/// Fetch latest release tag from GitHub using raw HTTPS + serde_json
fn get_latest_github_release() -> Result<String, Box<dyn Error>> {
    let host = "api.github.com";
    let path = format!("/repos/{}/{}/releases/latest", REPO_OWNER, REPO_NAME);

    // TCP + TLS connection
    let stream = TcpStream::connect((host, 443))?;
    let connector = TlsConnector::new()?;
    let mut stream = connector.connect(host, stream)?;

    // Send HTTP GET request
    let request = format!(
        "GET {} HTTP/1.1\r\n\
         Host: {}\r\n\
         User-Agent: git-selective-ignore-version-checker\r\n\
         Accept: application/json\r\n\
         Connection: close\r\n\r\n",
        path, host
    );
    stream.write_all(request.as_bytes())?;

    // Read response
    let mut reader = BufReader::new(stream);
    let mut body = String::new();
    let mut in_body = false;

    for line in reader.by_ref().lines() {
        let line = line?;
        if in_body {
            body.push_str(&line);
        } else if line.is_empty() {
            in_body = true; // blank line separates headers from body
        }
    }

    // Deserialize JSON
    let release: GitHubRelease = serde_json::from_str(&body)?;
    Ok(release.tag_name)
}

/// Normalize versions for comparison
fn normalize_version(version: &str) -> String {
    version
        .trim()
        .trim_start_matches(['v', 'V'])
        .chars()
        .filter(|c| c.is_ascii())
        .collect::<String>()
        .to_ascii_lowercase()
}

/// Run version check
pub fn run() {
    println!();
    println!("{}", "Version Check: ".cyan().bold());

    match get_local_version() {
        Ok(local_version) => {
            // Always print local version
            println!("├─ Local version: {}", local_version.bright_yellow().bold());

            // Try to get the latest version from GitHub
            match get_latest_github_release() {
                Ok(latest_version) => {
                    println!(
                        "├─ Latest GitHub release: {}",
                        latest_version.bright_green().bold()
                    );

                    let norm_local = normalize_version(&local_version);
                    let norm_latest = normalize_version(&latest_version);

                    if norm_local != norm_latest {
                        println!(
                            "└─ Update available! (Local: {}, Latest: {})",
                            local_version.red(),
                            latest_version.bright_green()
                        );
                    } else {
                        println!(
                            "{}",
                            "└─ You are running the latest version.".green().bold()
                        );
                    }
                }
                Err(_) => {
                    // Friendly message, not an error
                    println!(
                        "\n{}",
                        "Could not fetch release information from GitHub. \
                        This may be due to missing token, network issues, or no releases."
                            .bright_blue()
                            .bold()
                    );
                }
            }
        }
        Err(e) => {
            println!(
                "{}",
                format!("Could not determine local version: {}", e)
                    .red()
                    .bold()
            );
        }
    }
}
