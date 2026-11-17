use pumpkin_data::packet::CURRENT_MC_VERSION;
use pumpkin_util::text::click::ClickEvent;
use pumpkin_util::text::hover::HoverEvent;
use pumpkin_util::text::{TextComponent, color::NamedColor};
use pumpkin_util::translation::get_translation_text;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use pumpkin_util::PermissionLvl;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};

use crate::command::CommandSource;
use crate::command::argument_builder::{ArgumentBuilder, argument, command, literal};
use crate::command::argument_types::client_parsed::MessageArgumentType;
use crate::command::context::command_context::CommandContext;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};
use crate::telemetry::{
    HEADER_PUBLIC_KEY, HEADER_SIGNATURE, HEADER_TIMESTAMP, TelemetryClient,
    build_heartbeat_payload, resolve_identity_key, sign_telemetry_payload,
};

const DESCRIPTION: &str = "Display information about Pumpkin.";
const PERMISSION: &str = "pumpkin:command.pumpkin";
const ARG_SERVER_NAME: &str = "name";
const DEFAULT_ACCOUNTS_URL: &str = "https://accounts.pumpkinmc.org";

const CACHE_DURATION: Duration = Duration::from_hours(24);
const HTTP_TIMEOUT: Duration = Duration::from_secs(10);
const SPONSOR_URL: &str = "https://pumpkinmc.org/donate/";

struct Executor;
struct ContributorsExecutor;
struct LinkExecutor;
struct LinkNamedExecutor;

const CARGO_PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
const GIT_HASH: &str = env!("GIT_HASH");
const GIT_HASH_FULL: &str = env!("GIT_HASH_FULL");

#[derive(Deserialize, Clone)]
struct Contributor {
    login: String,
}

#[derive(Clone)]
struct ContributorCache {
    fetched_at: Instant,
    data: Vec<Contributor>,
}

static CONTRIBUTORS_CACHE: LazyLock<Mutex<Option<ContributorCache>>> =
    LazyLock::new(|| Mutex::new(None));

#[derive(Clone)]
struct SponsorCache {
    fetched_at: Instant,
    data: TextComponent,
}

static SPONSORS_CACHE: LazyLock<Mutex<Option<SponsorCache>>> = LazyLock::new(|| Mutex::new(None));

#[cfg(test)]
fn fetch_all_contributors_cached() -> Vec<Contributor> {
    if let Ok(guard) = CONTRIBUTORS_CACHE.lock()
        && let Some(cache) = guard.as_ref()
        && cache.fetched_at.elapsed() < CACHE_DURATION
    {
        return cache.data.clone();
    }

    Vec::new()
}

async fn fetch_all_contributors() -> Vec<Contributor> {
    let client = pumpkin_auth::client_builder()
        .user_agent("Pumpkin-MC")
        .timeout(HTTP_TIMEOUT)
        .build()
        .unwrap_or_default();

    let mut all_contributors = Vec::new();
    let mut next_url = Some(
        "https://api.github.com/repos/Pumpkin-MC/Pumpkin/contributors?per_page=100".to_string(),
    );

    while let Some(url) = next_url {
        let response = client.get(&url).send().await;

        match response {
            Ok(res) => {
                let link_header = res
                    .headers()
                    .get("link")
                    .and_then(|s| s.to_str().ok())
                    .map(String::from);

                if let Ok(contributors) = res.json::<Vec<Contributor>>().await {
                    all_contributors.extend(contributors);
                } else {
                    break;
                }

                next_url = link_header.as_deref().and_then(extract_next_url);
            }
            Err(_) => break,
        }
    }

    if all_contributors.is_empty() {
        return vec![];
    }

    all_contributors
}

fn extract_next_url(header: &str) -> Option<String> {
    header
        .split(',')
        .find(|part| part.contains("rel=\"next\""))
        .and_then(|part| {
            let start = part.find('<')? + 1;
            let end = part.find('>')?;
            Some(part[start..end].to_string())
        })
}

#[derive(Deserialize)]
struct Sponsor {
    name: String,
    tier: String,
    tier_slug: String,
}

#[derive(Deserialize)]
struct SponsorResponse {
    current: Vec<Sponsor>,
    #[allow(dead_code)]
    past: Vec<Sponsor>,
}

fn tier_priority(tier_slug: &str, tier_name: &str) -> u32 {
    let s = format!("{tier_slug} {tier_name}").to_lowercase();

    if s.contains("corporate platinum") || s.contains("corp platinum") {
        1
    } else if s.contains("corporate gold") || s.contains("corp gold") {
        2
    } else if s.contains("corporate silver") || s.contains("corp silver") {
        3
    } else if s.contains("corporate bronze") || s.contains("corp bronze") {
        4
    } else if s.contains("diamond") {
        5
    } else if s.contains("titanium") {
        6
    } else if s.contains("platinum") {
        7
    } else if s.contains("gold") {
        8
    } else if s.contains("silver") {
        9
    } else if s.contains("bronze") {
        10
    } else {
        11
    }
}

fn tier_color(tier_slug: &str, tier_name: &str) -> NamedColor {
    let s = format!("{tier_slug} {tier_name}").to_lowercase();

    if s.contains("diamond") {
        NamedColor::Aqua
    } else if s.contains("titanium") {
        NamedColor::DarkAqua
    } else if s.contains("platinum") {
        NamedColor::LightPurple
    } else if s.contains("gold") {
        NamedColor::Gold
    } else if s.contains("silver") {
        NamedColor::Gray
    } else if s.contains("bronze") {
        NamedColor::DarkRed
    } else {
        NamedColor::White
    }
}

async fn fetch_sponsors() -> TextComponent {
    let url = "https://market.pumpkinmc.org/api/v1/rest/donators";
    let client = pumpkin_auth::client_builder()
        .user_agent("Pumpkin-MC")
        .timeout(HTTP_TIMEOUT)
        .build()
        .unwrap_or_default();
    let response = client.get(url).send().await;

    let mut sponsors_text = TextComponent::text("Pumpkin Sponsors:\n")
        .color_named(NamedColor::Gold)
        .bold();

    if let Ok(res) = response
        && let Ok(data) = res.json::<SponsorResponse>().await
    {
        let mut current = data.current;
        current.sort_by(|a, b| {
            let prio_a = tier_priority(&a.tier_slug, &a.tier);
            let prio_b = tier_priority(&b.tier_slug, &b.tier);
            prio_a
                .cmp(&prio_b)
                .then_with(|| a.tier.cmp(&b.tier))
                .then_with(|| a.name.cmp(&b.name))
        });

        if current.is_empty() {
            sponsors_text = sponsors_text.add_child(
                TextComponent::text("No active sponsors at the moment.\n")
                    .color_named(NamedColor::White),
            );
        } else {
            let mut current_tier = String::new();
            for sponsor in current {
                if sponsor.tier != current_tier {
                    current_tier.clone_from(&sponsor.tier);
                    let color = tier_color(&sponsor.tier_slug, &sponsor.tier);
                    sponsors_text = sponsors_text.add_child(
                        TextComponent::text(format!("\n{current_tier}:\n"))
                            .color_named(color)
                            .bold(),
                    );
                }
                let color = tier_color(&sponsor.tier_slug, &sponsor.tier);
                sponsors_text = sponsors_text.add_child(
                    TextComponent::text(format!("  • {}\n", sponsor.name)).color_named(color),
                );
            }
        }

        sponsors_text = sponsors_text.add_child(
            TextComponent::text("\nSupport Pumpkin: ")
                .color_named(NamedColor::Gray)
                .add_child(
                    TextComponent::text("[Become a Sponsor]")
                        .click_event(ClickEvent::OpenUrl {
                            url: Cow::from(SPONSOR_URL),
                        })
                        .hover_event(HoverEvent::show_text(TextComponent::text(format!(
                            "Click to open {SPONSOR_URL}"
                        ))))
                        .color_named(NamedColor::Green)
                        .bold()
                        .underlined(),
                ),
        );

        return sponsors_text;
    }

    sponsors_text.add_child(
        TextComponent::text("Unable to load sponsors.\n")
            .color_named(NamedColor::Red)
            .add_child(
                TextComponent::text("[Become a Sponsor]")
                    .click_event(ClickEvent::OpenUrl {
                        url: Cow::from(SPONSOR_URL),
                    })
                    .hover_event(HoverEvent::show_text(TextComponent::text(format!(
                        "Click to open {SPONSOR_URL}"
                    ))))
                    .color_named(NamedColor::Green)
                    .bold()
                    .underlined(),
            ),
    )
}

#[cfg(test)]
fn fetch_sponsors_cached() -> TextComponent {
    if let Ok(guard) = SPONSORS_CACHE.lock()
        && let Some(cache) = guard.as_ref()
        && cache.fetched_at.elapsed() < CACHE_DURATION
    {
        return cache.data.clone();
    }

    TextComponent::text("Unable to load sponsors").color_named(NamedColor::Red)
}

#[expect(clippy::too_many_lines)]
impl CommandExecutor for Executor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let locale = context.source.output.get_locale();
        let profile = if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        };
        let version_string = format!("{CARGO_PKG_VERSION} (Commit: {GIT_HASH}/{profile})");
        let mut msg = TextComponent::text("");

        let version_translation = get_translation_text(
            "pumpkin:commands.pumpkin.version",
            locale,
            vec![TextComponent::text(version_string).0],
        );
        msg = msg.add_child(
            TextComponent::text(version_translation.clone())
                .hover_event(HoverEvent::show_text(TextComponent::text(format!(
                    "Commit: {GIT_HASH_FULL}\n\nClick to copy version\nRun /pumpkin contributors to see all contributors"
                ))))
                .click_event(ClickEvent::CopyToClipboard {
                    value: Cow::from(version_translation.replace('\n', "")),
                })
                .color_named(NamedColor::Green),
        );

        let desc_translation =
            get_translation_text("pumpkin:commands.pumpkin.description", locale, vec![]);
        let desc_hover_translation =
            get_translation_text("pumpkin:commands.pumpkin.description.hover", locale, vec![]);
        msg = msg.add_child(
            TextComponent::text(desc_translation.clone())
                .click_event(ClickEvent::CopyToClipboard {
                    value: Cow::from(desc_translation.replace('\n', "")),
                })
                .hover_event(HoverEvent::show_text(TextComponent::text(
                    desc_hover_translation,
                )))
                .color_named(NamedColor::White),
        );

        let mc_version_translation = get_translation_text(
            "pumpkin:commands.pumpkin.minecraft_version",
            locale,
            vec![
                TextComponent::text(CURRENT_MC_VERSION.to_string()).0,
                TextComponent::text(CURRENT_MC_VERSION.protocol_version().to_string()).0,
            ],
        );
        let mc_version_hover_translation = get_translation_text(
            "pumpkin:commands.pumpkin.minecraft_version.hover",
            locale,
            vec![],
        );
        msg = msg.add_child(
            TextComponent::text(mc_version_translation.clone())
                .click_event(ClickEvent::CopyToClipboard {
                    value: Cow::from(mc_version_translation.replace('\n', "")),
                })
                .hover_event(HoverEvent::show_text(TextComponent::text(
                    mc_version_hover_translation,
                )))
                .color_named(NamedColor::Gold),
        );

        let github_translation =
            get_translation_text("pumpkin:commands.pumpkin.github", locale, vec![]);
        let github_hover_translation =
            get_translation_text("pumpkin:commands.pumpkin.github.hover", locale, vec![]);
        msg = msg.add_child(
            TextComponent::text(github_translation)
                .click_event(ClickEvent::OpenUrl {
                    url: Cow::from("https://github.com/Pumpkin-MC/Pumpkin"),
                })
                .hover_event(HoverEvent::show_text(TextComponent::text(
                    github_hover_translation,
                )))
                .color_named(NamedColor::Blue)
                .bold()
                .underlined(),
        );

        msg = msg.add_child(TextComponent::text("  "));

        let website_translation =
            get_translation_text("pumpkin:commands.pumpkin.website", locale, vec![]);
        let website_hover_translation =
            get_translation_text("pumpkin:commands.pumpkin.website.hover", locale, vec![]);
        msg = msg.add_child(
            TextComponent::text(website_translation)
                .click_event(ClickEvent::OpenUrl {
                    url: Cow::from("https://pumpkinmc.org/"),
                })
                .hover_event(HoverEvent::show_text(TextComponent::text(
                    website_hover_translation,
                )))
                .color_named(NamedColor::Blue)
                .bold()
                .underlined(),
        );

        msg = msg.add_child(TextComponent::text("  "));

        msg = msg.add_child(
            TextComponent::text("[Sponsor]")
                .click_event(ClickEvent::OpenUrl {
                    url: Cow::from(SPONSOR_URL),
                })
                .hover_event(HoverEvent::show_text(TextComponent::text(format!(
                    "Click to open {SPONSOR_URL}\nRun /pumpkin contributors to see active sponsors"
                ))))
                .rainbow()
                .bold()
                .underlined(),
        );

        msg = msg.add_child(TextComponent::text("  "));

        msg = msg.add_child(
            TextComponent::text("[Contributors]")
                .click_event(ClickEvent::RunCommand {
                    command: Cow::from("/pumpkin contributors"),
                })
                .hover_event(HoverEvent::show_text(TextComponent::text(
                    "Click to run /pumpkin contributors",
                )))
                .color_named(NamedColor::Yellow)
                .bold()
                .underlined(),
        );

        context.source.send_message(msg);

        Ok(1)
    }
}

fn format_contributors_message(
    contributors: &[Contributor],
    sponsors_msg: TextComponent,
) -> TextComponent {
    if contributors.is_empty() {
        let mut msg =
            TextComponent::text("Unable to load contributors.\n\n").color_named(NamedColor::Red);
        msg = msg.add_child(sponsors_msg);
        return msg;
    }

    let count = contributors.len();
    let contributor_names = contributors
        .iter()
        .map(|c| c.login.as_str())
        .collect::<Vec<_>>()
        .join(", ");

    let mut msg = TextComponent::text(format!("Pumpkin Contributors ({count}):\n"))
        .color_named(NamedColor::Green)
        .bold();
    msg = msg.add_child(
        TextComponent::text(contributor_names)
            .gradient_named(&[NamedColor::DarkGreen, NamedColor::Green]),
    );

    msg = msg.add_child(TextComponent::text("\n\n"));
    msg = msg.add_child(sponsors_msg);
    msg
}

impl CommandExecutor for ContributorsExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let cached_contributors = CONTRIBUTORS_CACHE.lock().ok().and_then(|g| g.clone());
        let cached_sponsors = SPONSORS_CACHE.lock().ok().and_then(|g| g.clone());

        let has_fresh_contributors = cached_contributors
            .as_ref()
            .is_some_and(|c| c.fetched_at.elapsed() < CACHE_DURATION);
        let has_fresh_sponsors = cached_sponsors
            .as_ref()
            .is_some_and(|s| s.fetched_at.elapsed() < CACHE_DURATION);

        if has_fresh_contributors && has_fresh_sponsors {
            let contributors = cached_contributors.map(|c| c.data).unwrap_or_default();
            let sponsors_msg = cached_sponsors.map_or_else(
                || TextComponent::text("Unable to load sponsors"),
                |s| s.data,
            );
            let count = contributors.len();
            let msg = format_contributors_message(&contributors, sponsors_msg);
            context.source.send_message(msg);
            return Ok(count as i32);
        }

        if let (Some(c), Some(s)) = (&cached_contributors, &cached_sponsors) {
            let msg = format_contributors_message(&c.data, s.data.clone());
            context.source.send_message(msg);
        } else {
            context.source.send_message(
                TextComponent::text("Fetching Pumpkin contributors and sponsors...")
                    .color_named(NamedColor::Gray),
            );
        }

        let rt_handle = context
            .source
            .server
            .as_ref()
            .map(|s| s.runtime.clone())
            .or_else(|| tokio::runtime::Handle::try_current().ok());

        if let Some(handle) = rt_handle {
            let source = context.source.clone();
            let has_stale = cached_contributors.is_some() && cached_sponsors.is_some();
            handle.spawn(async move {
                let contributors = fetch_all_contributors().await;
                let sponsors_msg = fetch_sponsors().await;

                if !contributors.is_empty()
                    && let Ok(mut guard) = CONTRIBUTORS_CACHE.lock()
                {
                    *guard = Some(ContributorCache {
                        fetched_at: Instant::now(),
                        data: contributors.clone(),
                    });
                }
                if let Ok(mut guard) = SPONSORS_CACHE.lock() {
                    *guard = Some(SponsorCache {
                        fetched_at: Instant::now(),
                        data: sponsors_msg.clone(),
                    });
                }

                if !has_stale {
                    let msg = format_contributors_message(&contributors, sponsors_msg);
                    source.send_message(msg);
                }
            });
        }

        Ok(cached_contributors.map_or(0, |c| c.data.len() as i32))
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
struct ServerLinkInitRequest {
    server_name: String,
    server_type: String,
    server_version: String,
    minecraft_version: String,
    os: String,
    arch: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct ServerLinkInitResponse {
    code: String,
    link_url: String,
    expires_in_secs: u64,
}

#[derive(Deserialize)]
struct ApiErrorBody {
    error: String,
}

fn resolve_accounts_base_url() -> String {
    std::env::var("PUMPKIN_ACCOUNTS_URL")
        .ok()
        .map(|val| val.trim().trim_end_matches('/').to_string())
        .filter(|val| !val.is_empty())
        .unwrap_or_else(|| DEFAULT_ACCOUNTS_URL.to_string())
}

fn resolve_link_server_name(
    custom_name: Option<&str>,
    configured_name: Option<&str>,
    motd: Option<&str>,
) -> String {
    custom_name
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .or_else(|| configured_name.map(str::trim).filter(|s| !s.is_empty()))
        .or_else(|| motd.map(str::trim).filter(|s| !s.is_empty()))
        .unwrap_or("Pumpkin Server")
        .to_string()
}

fn detect_host_os() -> String {
    sysinfo::System::long_os_version().unwrap_or_else(|_| {
        let name = sysinfo::System::name().unwrap_or_else(|_| std::env::consts::OS.to_string());
        sysinfo::System::os_version().map_or(name.clone(), |ver| format!("{name} {ver}"))
    })
}

fn format_link_success_message(
    resp: &ServerLinkInitResponse,
    telemetry_enabled: bool,
) -> TextComponent {
    let expires_mins = resp.expires_in_secs.div_ceil(60);
    let mut msg = TextComponent::text("\nPumpkin Account Server Link\n")
        .color_named(NamedColor::Gold)
        .bold();

    msg = msg.add_child(TextComponent::text("  Code: ").color_named(NamedColor::Gray));
    msg = msg.add_child(
        TextComponent::text(resp.code.clone())
            .color_named(NamedColor::Aqua)
            .bold()
            .click_event(ClickEvent::CopyToClipboard {
                value: Cow::from(resp.code.clone()),
            })
            .hover_event(HoverEvent::show_text(TextComponent::text(
                "Click to copy link code",
            ))),
    );
    msg = msg.add_child(
        TextComponent::text(format!(" (expires in {expires_mins}m)\n"))
            .color_named(NamedColor::DarkGray),
    );

    msg = msg.add_child(TextComponent::text("  Claim: ").color_named(NamedColor::Gray));
    msg = msg.add_child(
        TextComponent::text(resp.link_url.clone())
            .color_named(NamedColor::Green)
            .bold()
            .underlined()
            .click_event(ClickEvent::OpenUrl {
                url: Cow::from(resp.link_url.clone()),
            })
            .hover_event(HoverEvent::show_text(TextComponent::text(
                "Click to open Pumpkin Account and claim this server",
            ))),
    );

    if !telemetry_enabled {
        msg = msg.add_child(
            TextComponent::text(
                "\n  Note: Telemetry is disabled in pumpkin.toml ([telemetry] enabled = false). Enable it to see live stats and crash reports.",
            )
            .color_named(NamedColor::Yellow),
        );
    }

    msg
}

async fn request_server_link(server_name: String) -> Result<ServerLinkInitResponse, String> {
    let signing_key = resolve_identity_key();
    let payload = ServerLinkInitRequest {
        server_name,
        server_type: "pumpkin".to_string(),
        server_version: CARGO_PKG_VERSION.to_string(),
        minecraft_version: CURRENT_MC_VERSION.to_string(),
        os: detect_host_os(),
        arch: std::env::consts::ARCH.to_string(),
    };

    let body_bytes =
        serde_json::to_vec(&payload).map_err(|e| format!("Failed to serialize payload: {e}"))?;
    let timestamp_str = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        .to_string();
    let (pubkey_hex, sig_hex) = sign_telemetry_payload(&signing_key, &timestamp_str, &body_bytes);

    let user_agent = format!("Pumpkin-Server/{CARGO_PKG_VERSION}");
    let client = pumpkin_auth::client_builder()
        .user_agent(&user_agent)
        .timeout(HTTP_TIMEOUT)
        .build()
        .map_err(|e| format!("Failed to initialize HTTP client: {e}"))?;

    let init_url = format!("{}/api/user/servers/link/init", resolve_accounts_base_url());
    let response = client
        .post(&init_url)
        .header("Content-Type", "application/json")
        .header(HEADER_PUBLIC_KEY, pubkey_hex)
        .header(HEADER_SIGNATURE, sig_hex)
        .header(HEADER_TIMESTAMP, timestamp_str)
        .body(body_bytes)
        .send()
        .await
        .map_err(|e| format!("Could not reach Pumpkin Account ({init_url}): {e}"))?;

    let status = response.status();
    if !status.is_success() {
        let detail = response
            .json::<ApiErrorBody>()
            .await
            .map_or_else(|_| format!("HTTP {status}"), |b| b.error);
        return Err(detail);
    }

    response
        .json::<ServerLinkInitResponse>()
        .await
        .map_err(|e| format!("Invalid response from Pumpkin Account: {e}"))
}

const fn is_console_only(source: &CommandSource) -> bool {
    source.output.is_console() && !source.silent
}

fn execute_link(context: &CommandContext, custom_name: Option<&str>) -> i32 {
    if !is_console_only(&context.source) {
        context.source.send_message(
            TextComponent::text(
                "For security, `pumpkin link` can only be executed from the server console.",
            )
            .color_named(NamedColor::Red),
        );
        return 0;
    }

    let server_ref = context.source.server.as_ref();
    let configured_name = server_ref.and_then(|s| s.telemetry_config.server_name.as_deref());
    let motd = server_ref.map(|s| s.advanced_config.networking.java.motd.as_str());
    let server_name = resolve_link_server_name(custom_name, configured_name, motd);

    let telemetry_enabled = server_ref.is_some_and(|s| s.telemetry_config.enabled);
    let immediate_heartbeat = server_ref.filter(|s| s.telemetry_config.enabled).map(|s| {
        (
            s.telemetry_config.endpoint.clone(),
            build_heartbeat_payload(s, &s.telemetry_config),
        )
    });

    let rt_handle = server_ref
        .map(|s| s.runtime.clone())
        .or_else(|| tokio::runtime::Handle::try_current().ok());

    let Some(handle) = rt_handle else {
        context.source.send_message(
            TextComponent::text("Unable to start async runtime for /pumpkin link.")
                .color_named(NamedColor::Red),
        );
        return 0;
    };

    context.source.send_message(
        TextComponent::text("Generating Pumpkin Account link code...")
            .color_named(NamedColor::Gray),
    );

    let source = context.source.clone();
    handle.spawn(async move {
        if let Some((endpoint, hb_payload)) = immediate_heartbeat
            && let Ok(http_client) = pumpkin_auth::client_builder()
                .user_agent(format!("Pumpkin-Server/{CARGO_PKG_VERSION}"))
                .timeout(HTTP_TIMEOUT)
                .build()
        {
            let client = TelemetryClient::new(resolve_identity_key(), http_client, endpoint);
            let _ = client.send_heartbeat(&hb_payload).await;
        }

        match request_server_link(server_name).await {
            Ok(resp) => {
                source.send_message(format_link_success_message(&resp, telemetry_enabled));
            }
            Err(err) => {
                source.send_message(
                    TextComponent::text(format!("Failed to generate server link code: {err}"))
                        .color_named(NamedColor::Red),
                );
            }
        }
    });

    1
}

impl CommandExecutor for LinkExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        Ok(execute_link(context, None))
    }
}

impl CommandExecutor for LinkNamedExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let custom_name = context.get_argument::<String>(ARG_SERVER_NAME)?;
        Ok(execute_link(context, Some(custom_name.as_str())))
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Zero),
    ));

    dispatcher.register_with_aliases(
        command("pumpkin", DESCRIPTION)
            .requires(PERMISSION)
            .then(literal("contributors").executes(ContributorsExecutor))
            .then(
                literal("link")
                    .requires(is_console_only)
                    .executes(LinkExecutor)
                    .then(
                        argument(ARG_SERVER_NAME, MessageArgumentType).executes(LinkNamedExecutor),
                    ),
            )
            .executes(Executor),
        &["version"],
    );
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn cache_duration_is_24_hours() {
        assert_eq!(CACHE_DURATION, Duration::from_hours(24));
    }

    #[test]
    fn contributor_cache_updates_and_retrieves() {
        let mut guard = CONTRIBUTORS_CACHE
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *guard = Some(ContributorCache {
            fetched_at: Instant::now(),
            data: vec![Contributor {
                login: "test_user".to_string(),
            }],
        });
        drop(guard);

        let contributors = fetch_all_contributors_cached();
        assert_eq!(contributors.len(), 1);
        assert_eq!(contributors[0].login, "test_user");
    }

    #[test]
    fn sponsor_cache_updates_and_retrieves() {
        let expected = TextComponent::text("Cached Sponsor Test");
        let mut guard = SPONSORS_CACHE
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *guard = Some(SponsorCache {
            fetched_at: Instant::now(),
            data: expected.clone(),
        });
        drop(guard);

        let cached = fetch_sponsors_cached();
        assert_eq!(cached, expected);
    }

    #[test]
    fn link_server_name_precedence() {
        assert_eq!(
            resolve_link_server_name(
                Some("  My Custom SMP "),
                Some("Telemetry Name"),
                Some("MOTD")
            ),
            "My Custom SMP"
        );
        assert_eq!(
            resolve_link_server_name(Some("   "), Some("Telemetry Name"), Some("MOTD")),
            "Telemetry Name"
        );
        assert_eq!(
            resolve_link_server_name(None, None, Some("  A blazingly fast server! ")),
            "A blazingly fast server!"
        );
        assert_eq!(
            resolve_link_server_name(None, Some(""), Some("")),
            "Pumpkin Server"
        );
    }

    #[test]
    fn link_payload_signing_verifies_with_telemetry_verifier() {
        let seed = [42u8; 32];
        let signing_key = ed25519_dalek::SigningKey::from_bytes(&seed);
        let req = ServerLinkInitRequest {
            server_name: "Survival Node 1".to_string(),
            server_type: "pumpkin".to_string(),
            server_version: CARGO_PKG_VERSION.to_string(),
            minecraft_version: CURRENT_MC_VERSION.to_string(),
            os: "Linux".to_string(),
            arch: "x86_64".to_string(),
        };
        let body = serde_json::to_vec(&req).unwrap();
        let ts = "1700000000";
        let (pub_hex, sig_hex) = sign_telemetry_payload(&signing_key, ts, &body);

        assert!(
            crate::telemetry::verify_telemetry_request(
                &pub_hex,
                &sig_hex,
                ts,
                &body,
                1_700_000_010
            )
            .is_ok()
        );
    }

    #[test]
    fn link_command_requires_non_silent_console() {
        let mut src = CommandSource::dummy();
        assert!(!is_console_only(&src));

        src.output = crate::command::CommandSender::Console;
        assert!(is_console_only(&src));

        src.silent = true;
        assert!(!is_console_only(&src));
    }
}
