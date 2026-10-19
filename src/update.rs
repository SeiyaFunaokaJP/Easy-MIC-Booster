//! Background check for a newer GitHub release.

use std::sync::{Arc, Mutex};
use std::thread;

pub const RELEASES_URL: &str = "https://github.com/SeiyaFunaokaJP/Easy-MIC-Booster/releases";
const API_URL: &str =
    "https://api.github.com/repos/SeiyaFunaokaJP/Easy-MIC-Booster/releases/latest";

#[derive(Clone, Debug, PartialEq)]
pub enum UpdateState {
    Idle,
    Checking,
    UpToDate,
    Available(String),
    Failed,
}

pub fn current_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

fn parse(v: &str) -> Vec<u64> {
    v.trim()
        .trim_start_matches(['v', 'V'])
        .split(['.', '-', '+'])
        .map_while(|p| p.parse().ok())
        .collect()
}

pub fn is_newer(latest: &str, current: &str) -> bool {
    let (a, b) = (parse(latest), parse(current));
    let n = a.len().max(b.len());
    let get = |v: &Vec<u64>, i: usize| v.get(i).copied().unwrap_or(0);
    (0..n)
        .map(|i| get(&a, i).cmp(&get(&b, i)))
        .find(|o| o.is_ne())
        == Some(std::cmp::Ordering::Greater)
}

pub fn check(state: Arc<Mutex<UpdateState>>, notify: Arc<dyn Fn() + Send + Sync>) {
    *state.lock().unwrap() = UpdateState::Checking;
    thread::Builder::new()
        .name("update-check".into())
        .spawn(move || {
            let result = (|| -> Result<String, Box<dyn std::error::Error>> {
                let agent = ureq::Agent::new_with_config(
                    ureq::config::Config::builder()
                        .tls_config(
                            ureq::tls::TlsConfig::builder()
                                .provider(if cfg!(windows) {
                                    ureq::tls::TlsProvider::NativeTls
                                } else {
                                    ureq::tls::TlsProvider::Rustls
                                })
                                .build(),
                        )
                        .build(),
                );
                let mut resp = agent
                    .get(API_URL)
                    .header(
                        "User-Agent",
                        concat!("EasyMICBooster/", env!("CARGO_PKG_VERSION")),
                    )
                    .header("Accept", "application/vnd.github+json")
                    .call()?;
                let json: serde_json::Value = resp.body_mut().read_json()?;
                Ok(json["tag_name"].as_str().unwrap_or_default().to_string())
            })();
            *state.lock().unwrap() = match result {
                Ok(tag) if is_newer(&tag, current_version()) => {
                    UpdateState::Available(tag.trim_start_matches('v').into())
                }
                Ok(_) => UpdateState::UpToDate,
                Err(e) => {
                    eprintln!("update check failed: {e}");
                    UpdateState::Failed
                }
            };
            notify();
        })
        .ok();
}

#[cfg(test)]
mod tests {
    use super::is_newer;

    #[test]
    fn compares_versions() {
        assert!(is_newer("v2.0.1", "2.0.0"));
        assert!(is_newer("v2.1", "2.0.9"));
        assert!(!is_newer("v2.0.0", "2.0.0"));
        assert!(!is_newer("v1.9.9", "2.0.0"));
    }
}
