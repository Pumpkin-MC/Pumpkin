use std::borrow::Cow;
use std::cmp::Ordering;
use std::sync::Arc;
use std::time::Duration;

use pumpkin_util::PermissionLvl;
use pumpkin_util::text::{TextComponent, click::ClickEvent, color::NamedColor, hover::HoverEvent};
use serde::Deserialize;
use tracing::{debug, warn};

use crate::entity::player::Player;
use crate::server::Server;

const CARGO_PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
const GITHUB_LATEST_RELEASE_URL: &str =
    "https://api.github.com/repos/Pumpkin-MC/Pumpkin/releases/latest";

/// Metadata for a GitHub release returned by the GitHub REST API.
#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct GitHubRelease {
    pub tag_name: String,
    pub html_url: String,
}

#[derive(Debug, PartialEq, Eq)]
struct ParsedVersion<'a> {
    major: u64,
    minor: u64,
    patch: u64,
    pre: Option<&'a str>,
    build: Option<&'a str>,
}

impl<'a> ParsedVersion<'a> {
    fn parse(raw: &'a str) -> Option<Self> {
        let trimmed = raw.trim();
        let without_prefix = trimmed.strip_prefix(['v', 'V']).unwrap_or(trimmed);
        let (version_and_pre, build) = match without_prefix.split_once('+') {
            Some((vp, b)) => (vp, Some(b)),
            None => (without_prefix, None),
        };
        let (core, pre) = match version_and_pre.split_once('-') {
            Some((c, p)) => (c, Some(p)),
            None => (version_and_pre, None),
        };

        let mut parts = core.split('.');
        let major = parts.next()?.parse::<u64>().ok()?;
        let minor = parts.next().unwrap_or("0").parse::<u64>().ok()?;
        let patch = parts.next().unwrap_or("0").parse::<u64>().ok()?;
        if parts.next().is_some() {
            return None;
        }

        Some(Self {
            major,
            minor,
            patch,
            pre,
            build,
        })
    }
}

fn compare_identifiers(a: &str, b: &str) -> Ordering {
    let mut iter_a = a.split(['.', '-']);
    let mut iter_b = b.split(['.', '-']);
    loop {
        match (iter_a.next(), iter_b.next()) {
            (Some(part_a), Some(part_b)) => {
                let ord = match (part_a.parse::<u64>(), part_b.parse::<u64>()) {
                    (Ok(num_a), Ok(num_b)) => num_a.cmp(&num_b),
                    _ => part_a.cmp(part_b),
                };
                if ord != Ordering::Equal {
                    return ord;
                }
            }
            (Some(_), None) => return Ordering::Greater,
            (None, Some(_)) => return Ordering::Less,
            (None, None) => return Ordering::Equal,
        }
    }
}

/// Returns `true` if `latest` represents a newer Pumpkin version than `current`.
#[must_use]
pub fn is_newer_version(current: &str, latest: &str) -> bool {
    let (Some(cur), Some(lat)) = (ParsedVersion::parse(current), ParsedVersion::parse(latest))
    else {
        return false;
    };

    let core_ord = (lat.major, lat.minor, lat.patch).cmp(&(cur.major, cur.minor, cur.patch));
    if core_ord != Ordering::Equal {
        return core_ord == Ordering::Greater;
    }

    let pre_ord = match (lat.pre, cur.pre) {
        (None, None) => Ordering::Equal,
        (None, Some(_)) => Ordering::Greater,
        (Some(_), None) => Ordering::Less,
        (Some(lat_pre), Some(cur_pre)) => compare_identifiers(lat_pre, cur_pre),
    };
    if pre_ord != Ordering::Equal {
        return pre_ord == Ordering::Greater;
    }

    if let (Some(lat_build), Some(cur_build)) = (lat.build, cur.build) {
        return compare_identifiers(lat_build, cur_build) == Ordering::Greater;
    }

    false
}

/// Queries the given GitHub release endpoint and returns the release if it is newer than `current_version`.
pub async fn check_for_updates_from_url(
    url: &str,
    current_version: &str,
    connect_timeout_ms: u32,
    read_timeout_ms: u32,
) -> Result<Option<GitHubRelease>, reqwest::Error> {
    let user_agent = format!("Pumpkin-Server/{CARGO_PKG_VERSION}");
    let client = pumpkin_auth::client_builder()
        .connect_timeout(Duration::from_millis(u64::from(connect_timeout_ms)))
        .timeout(Duration::from_millis(u64::from(read_timeout_ms)))
        .user_agent(&user_agent)
        .build()?;

    let release = client
        .get(url)
        .header(reqwest::header::ACCEPT, "application/vnd.github+json")
        .send()
        .await?
        .error_for_status()?
        .json::<GitHubRelease>()
        .await?;

    if is_newer_version(current_version, &release.tag_name) {
        Ok(Some(release))
    } else {
        Ok(None)
    }
}

/// Builds the chat message sent to operators when a new Pumpkin update is available.
#[must_use]
pub fn build_update_notification_message(release: &GitHubRelease) -> TextComponent {
    TextComponent::text("[Pumpkin] ")
        .color_named(NamedColor::Gold)
        .bold()
        .add_child(
            TextComponent::text("A new update is available: ").color_named(NamedColor::Yellow),
        )
        .add_child(
            TextComponent::text(release.tag_name.clone())
                .color_named(NamedColor::Green)
                .bold(),
        )
        .add_child(
            TextComponent::text(format!(" (current: {CARGO_PKG_VERSION}) "))
                .color_named(NamedColor::Gray),
        )
        .add_child(
            TextComponent::text("[Download]")
                .color_named(NamedColor::Aqua)
                .bold()
                .underlined()
                .click_event(ClickEvent::OpenUrl {
                    url: Cow::from(release.html_url.clone()),
                })
                .hover_event(HoverEvent::show_text(TextComponent::text(format!(
                    "Click to open {}",
                    release.html_url
                )))),
        )
}

/// Sends an update notification in chat if the player is an operator and an update was found.
pub fn notify_update_if_op(server: &Server, player: &Player) {
    if !server.advanced_config.update_checker.enabled
        || !server.advanced_config.update_checker.notify_ops_on_join
        || player.permission_lvl.load() <= PermissionLvl::Zero
    {
        return;
    }

    if let Some(release) = server.available_update.get() {
        player.send_system_message(&build_update_notification_message(release));
    }
}

/// Starts a background task to check for new Pumpkin updates on startup if enabled in configuration.
pub fn start_update_checker(server: &Arc<Server>) {
    let config = &server.advanced_config.update_checker;
    if !config.enabled {
        debug!("Update checker is disabled in configuration.");
        return;
    }

    let connect_timeout = config.connect_timeout;
    let read_timeout = config.read_timeout;
    let notify_ops = config.notify_ops_on_join;
    let server_task = server.clone();

    server.spawn_task(async move {
        let result = tokio::select! {
            res = check_for_updates_from_url(
                GITHUB_LATEST_RELEASE_URL,
                CARGO_PKG_VERSION,
                connect_timeout,
                read_timeout,
            ) => res,
            () = server_task.stop_token.cancelled() => return,
        };

        match result {
            Ok(Some(release)) => {
                warn!(
                    "A new version of Pumpkin is available: {} (current: {}). Download it at {}",
                    TextComponent::text(release.tag_name.clone())
                        .color_named(NamedColor::Green)
                        .to_pretty_console(),
                    TextComponent::text(CARGO_PKG_VERSION.to_string())
                        .color_named(NamedColor::Red)
                        .to_pretty_console(),
                    TextComponent::text(release.html_url.clone())
                        .color_named(NamedColor::Aqua)
                        .to_pretty_console()
                );

                if notify_ops {
                    let msg = build_update_notification_message(&release);
                    server_task.for_each_player(|player| {
                        if player.permission_lvl.load() > PermissionLvl::Zero {
                            player.send_system_message(&msg);
                        }
                    });
                }

                let _ = server_task.available_update.set(release);
            }
            Ok(None) => {
                debug!("Pumpkin is up to date ({CARGO_PKG_VERSION}).");
            }
            Err(err) => {
                debug!("Failed to check for Pumpkin updates: {err}");
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{Router, http::StatusCode, response::IntoResponse, routing::get};

    #[test]
    fn version_comparison() {
        assert!(is_newer_version("0.2.0+26.3-26.51", "0.2.1+26.3-26.51"));
        assert!(is_newer_version("0.2.1+26.3-26.51", "v0.3.0+26.3-26.51"));
        assert!(is_newer_version("0.2.1+26.3-26.51", "0.2.1+26.3-26.60"));
        assert!(is_newer_version("0.1.0-dev+26.2-26.45", "0.1.0+26.2-26.45"));

        assert!(!is_newer_version("0.2.1+26.3-26.60", "0.2.1+26.3-26.60"));
        assert!(!is_newer_version("0.2.1+26.3-26.60", "0.2.1+26.3-26.51"));
        assert!(!is_newer_version("0.2.1+26.3-26.60", "0.2.0+26.3-26.60"));
        assert!(!is_newer_version(
            "0.1.0+26.2-26.45",
            "0.1.0-dev+26.2-26.45"
        ));
        assert!(!is_newer_version("0.2.1", "nightly"));
    }

    #[tokio::test]
    async fn check_for_updates_http() {
        let _ = rustls::crypto::ring::default_provider().install_default();

        let app = Router::new().route(
            "/repos/Pumpkin-MC/Pumpkin/releases/latest",
            get(|| async {
                (
                    StatusCode::OK,
                    [(axum::http::header::CONTENT_TYPE, "application/json")],
                    r#"{"tag_name":"0.3.0+26.3-26.60","html_url":"https://github.com/Pumpkin-MC/Pumpkin/releases/tag/0.3.0%2B26.3-26.60"}"#,
                )
                    .into_response()
            }),
        );

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        let url = format!("http://127.0.0.1:{port}/repos/Pumpkin-MC/Pumpkin/releases/latest");

        let newer = check_for_updates_from_url(&url, "0.2.1+26.3-26.60", 5000, 5000)
            .await
            .unwrap();
        assert_eq!(
            newer,
            Some(GitHubRelease {
                tag_name: "0.3.0+26.3-26.60".to_string(),
                html_url: "https://github.com/Pumpkin-MC/Pumpkin/releases/tag/0.3.0%2B26.3-26.60"
                    .to_string(),
            })
        );

        let up_to_date = check_for_updates_from_url(&url, "0.3.0+26.3-26.60", 5000, 5000)
            .await
            .unwrap();
        assert_eq!(up_to_date, None);
    }
}
