//! Moonlight browser entrypoint.

use anyhow::{Context, Result};
use ml_engine_servo::ServoEngineBackend;
use std::env;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;
use url::Url;

/// Parse target URL from command-line arguments without unwrap on untrusted input.
pub fn parse_target_url(arg: Option<String>) -> Result<Url> {
    match arg {
        Some(raw) => {
            let parsed = if !raw.contains("://") {
                Url::parse(&format!("https://{raw}"))
            } else {
                Url::parse(&raw)
            };
            let url = parsed.context("invalid URL provided as command-line argument")?;
            match url.scheme() {
                "http" | "https" | "about" | "moonlight" => Ok(url),
                unsupported => anyhow::bail!("unsupported URL scheme: {unsupported}"),
            }
        }
        None => Url::parse("https://duckduckgo.com").context("failed to parse default URL"),
    }
}

fn main() -> Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    let _ = tracing::subscriber::set_global_default(subscriber);

    let raw_arg = env::args().nth(1);

    if let Some(arg) = raw_arg.as_deref() {
        match arg {
            "--help" | "-h" => {
                println!(
                    "Moonlight browser\n\nUsage: moonlight [URL]\n\nArguments:\n  [URL]  The URL or domain to open (default: https://duckduckgo.com)\n\nOptions:\n  -h, --help     Print help information\n  -V, --version  Print version information"
                );
                return Ok(());
            }
            "--version" | "-V" => {
                println!("Moonlight {}", env!("CARGO_PKG_VERSION"));
                return Ok(());
            }
            _ => {}
        }
    }

    let target_url = parse_target_url(raw_arg)?;
    info!("Starting Moonlight browser with target URL: {target_url}");

    let backend = ServoEngineBackend::new();
    if let Err(e) = ml_shell::run(backend, target_url) {
        anyhow::bail!("browser execution error: {e}");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_default_url() {
        let url = parse_target_url(None).expect("default URL must be valid");
        assert_eq!(url.as_str(), "https://duckduckgo.com/");
    }

    #[test]
    fn test_parse_explicit_url() {
        let url = parse_target_url(Some("https://example.com/test".to_string()))
            .expect("explicit HTTPS URL is valid");
        assert_eq!(url.as_str(), "https://example.com/test");
    }

    #[test]
    fn test_parse_bare_domain() {
        let url =
            parse_target_url(Some("example.com".to_string())).expect("bare domain prepends https");
        assert_eq!(url.as_str(), "https://example.com/");
    }

    #[test]
    fn test_parse_invalid_scheme() {
        let res = parse_target_url(Some("ftp://ftp.example.com".to_string()));
        assert!(res.is_err());
    }
}
