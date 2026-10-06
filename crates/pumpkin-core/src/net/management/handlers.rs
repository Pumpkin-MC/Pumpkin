use std::{
    net::IpAddr,
    str::FromStr,
    sync::{Arc, LazyLock, atomic::Ordering},
};

use pumpkin_config::{op::Op, whitelist::WhitelistEntry};
use pumpkin_data::{
    game_rules::{GameRule, GameRuleValue},
    packet::CURRENT_MC_VERSION,
    translation,
};
use pumpkin_util::permission::PermissionLvl;
use serde::Deserialize;
use serde_json::{Value, json};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::{
    command::commands::kick_non_whitelisted_players,
    data::{
        SaveJSONConfiguration,
        banlist_serializer::{BannedIpEntry, BannedPlayerEntry},
    },
    net::{DisconnectReason, GameProfile},
    server::Server,
};

use super::{
    dto::{
        IncomingIpBanDto, IpBanDto, KickPlayerDto, OperatorDto, PlayerDto, ServerState,
        SystemMessageDto, TypedGameRule, UntypedGameRule, UserBanDto, VersionDto,
    },
    rpc::{RpcError, RpcRequest, RpcResponse, is_valid_request_id},
};

#[expect(
    clippy::expect_used,
    reason = "assets/json-rpc-api-schema.json is validated at compile/init time"
)]
static OPENRPC_SCHEMA: LazyLock<Value> = LazyLock::new(|| {
    serde_json::from_str(include_str!(
        "../../../../../assets/json-rpc-api-schema.json"
    ))
    .expect("assets/json-rpc-api-schema.json must be valid JSON")
});

fn extract_param<'a>(params: Option<&'a Value>, param_name: &str) -> Result<&'a Value, RpcError> {
    match params {
        Some(Value::Object(map)) => map.get(param_name).ok_or_else(|| {
            RpcError::invalid_params(Some(format!(
                "Params passed by-name, but expected param [{param_name}] does not exist"
            )))
        }),
        Some(Value::Array(arr)) => {
            if arr.len() == 1 {
                Ok(&arr[0])
            } else {
                Err(RpcError::invalid_params(Some(
                    "Expected exactly one element in the params array".to_string(),
                )))
            }
        }
        _ => Err(RpcError::invalid_params(Some(
            "Expected params as array or named".to_string(),
        ))),
    }
}

fn check_no_params(params: Option<&Value>) -> Result<(), RpcError> {
    match params {
        None | Some(Value::Null) => Ok(()),
        Some(Value::Array(arr)) if arr.is_empty() => Ok(()),
        _ => Err(RpcError::invalid_params(Some(
            "Expected no params, or an empty array".to_string(),
        ))),
    }
}

fn parse_param<T: for<'de> Deserialize<'de>>(
    params: Option<&Value>,
    param_name: &str,
) -> Result<T, RpcError> {
    let val = extract_param(params, param_name)?;
    serde_json::from_value(val.clone())
        .map_err(|e| RpcError::invalid_params(Some(format!("Failed to parse parameter: {e}"))))
}

const fn u8_to_permission_lvl(lvl: u8) -> PermissionLvl {
    match lvl {
        0 => PermissionLvl::Zero,
        1 => PermissionLvl::One,
        2 => PermissionLvl::Two,
        3 => PermissionLvl::Three,
        _ => PermissionLvl::Four,
    }
}

pub async fn dispatch(request: RpcRequest, server: &Arc<Server>) -> Option<RpcResponse> {
    let id = request.id.clone();
    if let Some(ref req_id) = id
        && !is_valid_request_id(req_id)
    {
        return Some(RpcResponse::error(
            Value::Null,
            RpcError::invalid_request(Some(
                "Invalid request id - only String, Number and NULL supported".to_string(),
            )),
        ));
    }

    let Some(raw_method) = request.method else {
        return id.map(|req_id| {
            RpcResponse::error(
                req_id,
                RpcError::invalid_request(Some("Missing method name".to_string())),
            )
        });
    };

    let method = normalize_method(&raw_method);
    let result = execute_method(&method, request.params.as_ref(), server).await;

    id.map(|req_id| match result {
        Ok(val) => RpcResponse::success(req_id, val),
        Err(err) => RpcResponse::error(req_id, err),
    })
}

fn normalize_method(method: &str) -> String {
    if method == "rpc.discover" || method.contains(':') {
        method.to_string()
    } else {
        format!("minecraft:{method}")
    }
}

#[expect(clippy::too_many_lines)]
async fn execute_method(
    method: &str,
    params: Option<&Value>,
    server: &Arc<Server>,
) -> Result<Value, RpcError> {
    match method {
        "rpc.discover" => {
            check_no_params(params)?;
            Ok(OPENRPC_SCHEMA.clone())
        }

        // ==========================================
        // Allowlist
        // ==========================================
        "minecraft:allowlist" => {
            check_no_params(params)?;
            let list = get_allowlist(server);
            Ok(serde_json::to_value(list).unwrap_or(Value::Null))
        }
        "minecraft:allowlist/add" => {
            let to_add: Vec<PlayerDto> = parse_param(params, "add")?;
            {
                let mut whitelist_guard = server
                    .data
                    .whitelist_config
                    .write()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                for p in &to_add {
                    let uuid = p.id.unwrap_or_else(Uuid::new_v4);
                    let name = p.name.clone().unwrap_or_default();
                    if !whitelist_guard.whitelist.iter().any(|e| e.uuid == uuid) {
                        whitelist_guard
                            .whitelist
                            .push(WhitelistEntry::new(uuid, name));
                        server.management_hub.broadcast_player_added_to_allowlist(p);
                    }
                }
                whitelist_guard.save();
            };
            Ok(serde_json::to_value(get_allowlist(server)).unwrap_or(Value::Null))
        }
        "minecraft:allowlist/remove" => {
            let to_remove: Vec<PlayerDto> = parse_param(params, "remove")?;
            {
                let mut whitelist_guard = server
                    .data
                    .whitelist_config
                    .write()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                for p in &to_remove {
                    if let Some(pos) = whitelist_guard.whitelist.iter().position(|e| {
                        p.id.is_some_and(|id| e.uuid == id)
                            || p.name.as_ref().is_some_and(|name| &e.name == name)
                    }) {
                        whitelist_guard.whitelist.remove(pos);
                        server
                            .management_hub
                            .broadcast_player_removed_from_allowlist(p);
                    }
                }
                whitelist_guard.save();
            };
            kick_non_whitelisted_players(server);
            Ok(serde_json::to_value(get_allowlist(server)).unwrap_or(Value::Null))
        }
        "minecraft:allowlist/clear" => {
            check_no_params(params)?;
            {
                let mut whitelist_guard = server
                    .data
                    .whitelist_config
                    .write()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                for entry in whitelist_guard.whitelist.drain(..) {
                    let dto = PlayerDto::new(Some(entry.uuid), Some(entry.name));
                    server
                        .management_hub
                        .broadcast_player_removed_from_allowlist(&dto);
                }
                whitelist_guard.save();
            };
            kick_non_whitelisted_players(server);
            Ok(json!([]))
        }
        "minecraft:allowlist/set" => {
            let new_list: Vec<PlayerDto> = parse_param(params, "players")?;
            {
                let mut whitelist_guard = server
                    .data
                    .whitelist_config
                    .write()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);

                for old in &whitelist_guard.whitelist {
                    if !new_list
                        .iter()
                        .any(|n| n.id.is_some_and(|id| id == old.uuid))
                    {
                        let dto = PlayerDto::new(Some(old.uuid), Some(old.name.clone()));
                        server
                            .management_hub
                            .broadcast_player_removed_from_allowlist(&dto);
                    }
                }

                let mut final_entries = Vec::new();
                for p in &new_list {
                    let uuid = p.id.unwrap_or_else(Uuid::new_v4);
                    let name = p.name.clone().unwrap_or_default();
                    if !whitelist_guard.whitelist.iter().any(|e| e.uuid == uuid) {
                        server.management_hub.broadcast_player_added_to_allowlist(p);
                    }
                    final_entries.push(WhitelistEntry::new(uuid, name));
                }

                whitelist_guard.whitelist = final_entries;
                whitelist_guard.save();
            };
            kick_non_whitelisted_players(server);
            Ok(serde_json::to_value(get_allowlist(server)).unwrap_or(Value::Null))
        }

        // ==========================================
        // Bans
        // ==========================================
        "minecraft:bans" => {
            check_no_params(params)?;
            Ok(serde_json::to_value(get_bans(server)).unwrap_or(Value::Null))
        }
        "minecraft:bans/add" => {
            let add_bans: Vec<UserBanDto> = parse_param(params, "add")?;
            {
                let mut ban_guard = server
                    .data
                    .banned_player_list
                    .write()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                for ban in &add_bans {
                    let uuid = ban.player.id.unwrap_or_else(Uuid::new_v4);
                    let name = ban.player.name.clone().unwrap_or_default();
                    let profile = GameProfile {
                        id: uuid,
                        name: name.clone(),
                        properties: arc_swap::ArcSwap::from_pointee(Vec::new()),
                        profile_actions: None,
                    };
                    let expires = ban.expires.as_deref().and_then(|exp| {
                        OffsetDateTime::parse(exp, &time::format_description::well_known::Rfc3339)
                            .ok()
                    });
                    let entry = BannedPlayerEntry::new(
                        &profile,
                        ban.source
                            .clone()
                            .unwrap_or_else(|| "Management server".to_string()),
                        expires,
                        ban.reason.clone().unwrap_or_default(),
                    );
                    ban_guard.banned_players.retain(|b| b.uuid != uuid);
                    ban_guard.banned_players.push(entry);
                    server.management_hub.broadcast_player_banned(ban);

                    for player in server.get_all_players() {
                        if player.gameprofile.id == uuid {
                            player.kick(
                                DisconnectReason::Kicked,
                                &pumpkin_macros::translate_cross!(
                                    translation::java::MULTIPLAYER_DISCONNECT_BANNED,
                                    translation::bedrock::DISCONNECT_KICKED
                                ),
                            );
                        }
                    }
                }
                ban_guard.save();
            };
            Ok(serde_json::to_value(get_bans(server)).unwrap_or(Value::Null))
        }
        "minecraft:bans/remove" => {
            let to_remove: Vec<PlayerDto> = parse_param(params, "remove")?;
            {
                let mut ban_guard = server
                    .data
                    .banned_player_list
                    .write()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                for p in &to_remove {
                    if let Some(pos) = ban_guard.banned_players.iter().position(|b| {
                        p.id.is_some_and(|id| b.uuid == id)
                            || p.name.as_ref().is_some_and(|name| &b.name == name)
                    }) {
                        ban_guard.banned_players.remove(pos);
                        server.management_hub.broadcast_player_unbanned(p);
                    }
                }
                ban_guard.save();
            };
            Ok(serde_json::to_value(get_bans(server)).unwrap_or(Value::Null))
        }
        "minecraft:bans/clear" => {
            check_no_params(params)?;
            {
                let mut ban_guard = server
                    .data
                    .banned_player_list
                    .write()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                for b in ban_guard.banned_players.drain(..) {
                    let dto = PlayerDto::new(Some(b.uuid), Some(b.name));
                    server.management_hub.broadcast_player_unbanned(&dto);
                }
                ban_guard.save();
            };
            Ok(json!([]))
        }
        "minecraft:bans/set" => {
            let new_bans: Vec<UserBanDto> = parse_param(params, "bans")?;
            {
                let mut ban_guard = server
                    .data
                    .banned_player_list
                    .write()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);

                for old in &ban_guard.banned_players {
                    if !new_bans
                        .iter()
                        .any(|n| n.player.id.is_some_and(|id| id == old.uuid))
                    {
                        let dto = PlayerDto::new(Some(old.uuid), Some(old.name.clone()));
                        server.management_hub.broadcast_player_unbanned(&dto);
                    }
                }

                let mut final_bans = Vec::new();
                for ban in &new_bans {
                    let uuid = ban.player.id.unwrap_or_else(Uuid::new_v4);
                    let name = ban.player.name.clone().unwrap_or_default();
                    let profile = GameProfile {
                        id: uuid,
                        name: name.clone(),
                        properties: arc_swap::ArcSwap::from_pointee(Vec::new()),
                        profile_actions: None,
                    };
                    let expires = ban.expires.as_deref().and_then(|exp| {
                        OffsetDateTime::parse(exp, &time::format_description::well_known::Rfc3339)
                            .ok()
                    });
                    final_bans.push(BannedPlayerEntry::new(
                        &profile,
                        ban.source
                            .clone()
                            .unwrap_or_else(|| "Management server".to_string()),
                        expires,
                        ban.reason.clone().unwrap_or_default(),
                    ));
                    if !ban_guard.banned_players.iter().any(|b| b.uuid == uuid) {
                        server.management_hub.broadcast_player_banned(ban);
                        for player in server.get_all_players() {
                            if player.gameprofile.id == uuid {
                                player.kick(
                                    DisconnectReason::Kicked,
                                    &pumpkin_macros::translate_cross!(
                                        translation::java::MULTIPLAYER_DISCONNECT_BANNED,
                                        translation::bedrock::DISCONNECT_KICKED
                                    ),
                                );
                            }
                        }
                    }
                }

                ban_guard.banned_players = final_bans;
                ban_guard.save();
            };
            Ok(serde_json::to_value(get_bans(server)).unwrap_or(Value::Null))
        }

        // ==========================================
        // IP Bans
        // ==========================================
        "minecraft:ip_bans" => {
            check_no_params(params)?;
            Ok(serde_json::to_value(get_ip_bans(server)).unwrap_or(Value::Null))
        }
        "minecraft:ip_bans/add" => {
            let add_bans: Vec<IncomingIpBanDto> = parse_param(params, "add")?;
            {
                let mut ip_ban_guard = server
                    .data
                    .banned_ip_list
                    .write()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                for ban in &add_bans {
                    let target_ip: Option<IpAddr> = ban.ip.as_ref().map_or_else(
                        || {
                            ban.player.as_ref().and_then(|player_dto| {
                                server
                                    .get_all_players()
                                    .into_iter()
                                    .find(|p| {
                                        player_dto.id.is_some_and(|id| id == p.gameprofile.id)
                                            || player_dto
                                                .name
                                                .as_ref()
                                                .is_some_and(|name| name == &p.gameprofile.name)
                                    })
                                    .map(|p| p.client.address().ip())
                            })
                        },
                        |ip_str| ip_str.parse().ok(),
                    );

                    if let Some(ip) = target_ip {
                        let expires = ban.expires.as_deref().and_then(|exp| {
                            OffsetDateTime::parse(
                                exp,
                                &time::format_description::well_known::Rfc3339,
                            )
                            .ok()
                        });
                        let entry = BannedIpEntry::new(
                            ip,
                            ban.source
                                .clone()
                                .unwrap_or_else(|| "Management server".to_string()),
                            expires,
                            ban.reason.clone().unwrap_or_default(),
                        );
                        ip_ban_guard.banned_ips.retain(|b| b.ip != ip);
                        ip_ban_guard.banned_ips.push(entry);

                        let dto = IpBanDto {
                            ip: Some(ip.to_string()),
                            reason: ban.reason.clone(),
                            source: ban.source.clone(),
                            expires: ban.expires.clone(),
                        };
                        server.management_hub.broadcast_ip_banned(&dto);

                        for player in server.get_all_players() {
                            if player.client.address().ip() == ip {
                                player.kick(
                                    DisconnectReason::Kicked,
                                    &pumpkin_macros::translate_cross!(
                                        translation::java::MULTIPLAYER_DISCONNECT_BANNED_IP_REASON,
                                        translation::bedrock::DISCONNECT_KICKED
                                    ),
                                );
                            }
                        }
                    }
                }
                ip_ban_guard.save();
            };
            Ok(serde_json::to_value(get_ip_bans(server)).unwrap_or(Value::Null))
        }
        "minecraft:ip_bans/remove" => {
            let ips: Vec<String> = parse_param(params, "ip")?;
            {
                let mut ip_ban_guard = server
                    .data
                    .banned_ip_list
                    .write()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                for ip_str in &ips {
                    if let Ok(ip) = ip_str.parse::<IpAddr>() {
                        ip_ban_guard.banned_ips.retain(|b| b.ip != ip);
                        server.management_hub.broadcast_ip_unbanned(ip_str);
                    }
                }
                ip_ban_guard.save();
            };
            Ok(serde_json::to_value(get_ip_bans(server)).unwrap_or(Value::Null))
        }
        "minecraft:ip_bans/clear" => {
            check_no_params(params)?;
            {
                let mut ip_ban_guard = server
                    .data
                    .banned_ip_list
                    .write()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                for b in ip_ban_guard.banned_ips.drain(..) {
                    server
                        .management_hub
                        .broadcast_ip_unbanned(&b.ip.to_string());
                }
                ip_ban_guard.save();
            };
            Ok(json!([]))
        }
        "minecraft:ip_bans/set" => {
            let new_bans: Vec<IpBanDto> = parse_param(params, "banlist")?;
            {
                let mut ip_ban_guard = server
                    .data
                    .banned_ip_list
                    .write()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);

                for old in &ip_ban_guard.banned_ips {
                    let old_str = old.ip.to_string();
                    if !new_bans
                        .iter()
                        .any(|n| n.ip.as_ref().is_some_and(|ip| ip == &old_str))
                    {
                        server.management_hub.broadcast_ip_unbanned(&old_str);
                    }
                }

                let mut final_bans = Vec::new();
                for ban in &new_bans {
                    if let Some(ref ip_str) = ban.ip
                        && let Ok(ip) = ip_str.parse::<IpAddr>()
                    {
                        let expires = ban.expires.as_deref().and_then(|exp| {
                            OffsetDateTime::parse(
                                exp,
                                &time::format_description::well_known::Rfc3339,
                            )
                            .ok()
                        });
                        final_bans.push(BannedIpEntry::new(
                            ip,
                            ban.source
                                .clone()
                                .unwrap_or_else(|| "Management server".to_string()),
                            expires,
                            ban.reason.clone().unwrap_or_default(),
                        ));
                        if !ip_ban_guard.banned_ips.iter().any(|b| b.ip == ip) {
                            server.management_hub.broadcast_ip_banned(ban);
                            for player in server.get_all_players() {
                                if player.client.address().ip() == ip {
                                    player.kick(
                                            DisconnectReason::Kicked,
                                            &pumpkin_macros::translate_cross!(
                                                translation::java::MULTIPLAYER_DISCONNECT_BANNED_IP_REASON,
                                                translation::bedrock::DISCONNECT_KICKED
                                            ),
                                        );
                                }
                            }
                        }
                    }
                }

                ip_ban_guard.banned_ips = final_bans;
                ip_ban_guard.save();
            };
            Ok(serde_json::to_value(get_ip_bans(server)).unwrap_or(Value::Null))
        }

        // ==========================================
        // Players
        // ==========================================
        "minecraft:players" => {
            check_no_params(params)?;
            let players: Vec<PlayerDto> = server
                .get_all_players()
                .into_iter()
                .map(|p| PlayerDto::new(Some(p.gameprofile.id), Some(p.gameprofile.name.clone())))
                .collect();
            Ok(serde_json::to_value(players).unwrap_or(Value::Null))
        }
        "minecraft:players/kick" => {
            let kick_requests: Vec<KickPlayerDto> = parse_param(params, "kick")?;
            let mut kicked = Vec::new();
            for req in kick_requests {
                let target = server.get_all_players().into_iter().find(|p| {
                    req.player.id.is_some_and(|id| id == p.gameprofile.id)
                        || req
                            .player
                            .name
                            .as_ref()
                            .is_some_and(|name| name == &p.gameprofile.name)
                });
                if let Some(player) = target {
                    let message = req.message.as_ref().map_or_else(
                        || {
                            pumpkin_macros::translate_cross!(
                                translation::java::MULTIPLAYER_DISCONNECT_KICKED,
                                translation::bedrock::DISCONNECT_KICKED
                            )
                        },
                        super::dto::MessageDto::to_text_component,
                    );
                    player.kick(DisconnectReason::Kicked, &message);
                    kicked.push(req.player);
                }
            }
            Ok(serde_json::to_value(kicked).unwrap_or(Value::Null))
        }

        // ==========================================
        // Operators
        // ==========================================
        "minecraft:operators" => {
            check_no_params(params)?;
            Ok(serde_json::to_value(get_operators(server)).unwrap_or(Value::Null))
        }
        "minecraft:operators/add" => {
            let add_ops: Vec<OperatorDto> = parse_param(params, "add")?;
            {
                let mut op_guard = server
                    .data
                    .operator_config
                    .write()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                for op in &add_ops {
                    let uuid = op.player.id.unwrap_or_else(Uuid::new_v4);
                    let name = op.player.name.clone().unwrap_or_default();
                    let level = op
                        .permission_level
                        .map_or(PermissionLvl::Four, u8_to_permission_lvl);
                    let bypass = op.bypasses_player_limit.unwrap_or(false);
                    op_guard.ops.retain(|o| o.uuid != uuid);
                    op_guard.ops.push(Op::new(uuid, name, level, bypass));
                    server.management_hub.broadcast_player_oped(op);

                    for player in server.get_all_players() {
                        if player.gameprofile.id == uuid {
                            player.permission_lvl.store(level);
                        }
                    }
                }
                op_guard.save();
            };
            Ok(serde_json::to_value(get_operators(server)).unwrap_or(Value::Null))
        }
        "minecraft:operators/remove" => {
            let to_remove: Vec<PlayerDto> = parse_param(params, "remove")?;
            {
                let mut op_guard = server
                    .data
                    .operator_config
                    .write()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                for p in &to_remove {
                    if let Some(pos) = op_guard.ops.iter().position(|o| {
                        p.id.is_some_and(|id| o.uuid == id)
                            || p.name.as_ref().is_some_and(|name| &o.name == name)
                    }) {
                        let removed = op_guard.ops.remove(pos);
                        let dto = OperatorDto {
                            player: PlayerDto::new(Some(removed.uuid), Some(removed.name)),
                            permission_level: Some(removed.level as u8),
                            bypasses_player_limit: Some(removed.bypasses_player_limit),
                        };
                        server.management_hub.broadcast_player_deoped(&dto);

                        for player in server.get_all_players() {
                            if player.gameprofile.id == removed.uuid {
                                player.permission_lvl.store(PermissionLvl::Zero);
                            }
                        }
                    }
                }
                op_guard.save();
            };
            Ok(serde_json::to_value(get_operators(server)).unwrap_or(Value::Null))
        }
        "minecraft:operators/clear" => {
            check_no_params(params)?;
            {
                let mut op_guard = server
                    .data
                    .operator_config
                    .write()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                for removed in op_guard.ops.drain(..) {
                    let dto = OperatorDto {
                        player: PlayerDto::new(Some(removed.uuid), Some(removed.name)),
                        permission_level: Some(removed.level as u8),
                        bypasses_player_limit: Some(removed.bypasses_player_limit),
                    };
                    server.management_hub.broadcast_player_deoped(&dto);

                    for player in server.get_all_players() {
                        if player.gameprofile.id == removed.uuid {
                            player.permission_lvl.store(PermissionLvl::Zero);
                        }
                    }
                }
                op_guard.save();
            };
            Ok(json!([]))
        }
        "minecraft:operators/set" => {
            let new_ops: Vec<OperatorDto> = parse_param(params, "operators")?;
            {
                let mut op_guard = server
                    .data
                    .operator_config
                    .write()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);

                for old in &op_guard.ops {
                    if !new_ops
                        .iter()
                        .any(|n| n.player.id.is_some_and(|id| id == old.uuid))
                    {
                        let dto = OperatorDto {
                            player: PlayerDto::new(Some(old.uuid), Some(old.name.clone())),
                            permission_level: Some(old.level as u8),
                            bypasses_player_limit: Some(old.bypasses_player_limit),
                        };
                        server.management_hub.broadcast_player_deoped(&dto);
                        for player in server.get_all_players() {
                            if player.gameprofile.id == old.uuid {
                                player.permission_lvl.store(PermissionLvl::Zero);
                            }
                        }
                    }
                }

                let mut final_ops = Vec::new();
                for op in &new_ops {
                    let uuid = op.player.id.unwrap_or_else(Uuid::new_v4);
                    let name = op.player.name.clone().unwrap_or_default();
                    let level = op
                        .permission_level
                        .map_or(PermissionLvl::Four, u8_to_permission_lvl);
                    let bypass = op.bypasses_player_limit.unwrap_or(false);
                    final_ops.push(Op::new(uuid, name, level, bypass));

                    if !op_guard.ops.iter().any(|o| o.uuid == uuid) {
                        server.management_hub.broadcast_player_oped(op);
                    }
                    for player in server.get_all_players() {
                        if player.gameprofile.id == uuid {
                            player.permission_lvl.store(level);
                        }
                    }
                }

                op_guard.ops = final_ops;
                op_guard.save();
            };
            Ok(serde_json::to_value(get_operators(server)).unwrap_or(Value::Null))
        }

        // ==========================================
        // Server State
        // ==========================================
        "minecraft:server/status" => {
            check_no_params(params)?;
            let state = ServerState {
                started: true,
                players: server
                    .get_all_players()
                    .into_iter()
                    .map(|p| {
                        PlayerDto::new(Some(p.gameprofile.id), Some(p.gameprofile.name.clone()))
                    })
                    .collect(),
                version: VersionDto {
                    name: CURRENT_MC_VERSION.to_string(),
                    protocol: CURRENT_MC_VERSION.protocol_version(),
                },
            };
            Ok(serde_json::to_value(state).unwrap_or(Value::Null))
        }
        "minecraft:server/save" => {
            let _flush: bool = parse_param(params, "flush")?;
            server.management_hub.broadcast_server_saving();
            let _ = server.save_all().await;
            server.management_hub.broadcast_server_saved();
            Ok(json!(true))
        }
        "minecraft:server/stop" => {
            check_no_params(params)?;
            server.management_hub.broadcast_server_stopping();
            let s = server.clone();
            tokio::spawn(async move {
                s.shutdown().await;
                std::process::exit(0);
            });
            Ok(json!(true))
        }
        "minecraft:server/system_message" => {
            let sys_msg: SystemMessageDto = parse_param(params, "message")?;
            let component = sys_msg.message.to_text_component();
            let overlay = sys_msg.overlay;
            if let Some(ref receivers) = sys_msg.receiving_players {
                if receivers.is_empty() {
                    return Ok(json!(false));
                }
                for player_dto in receivers {
                    let target = server.get_all_players().into_iter().find(|p| {
                        player_dto.id.is_some_and(|id| id == p.gameprofile.id)
                            || player_dto
                                .name
                                .as_ref()
                                .is_some_and(|name| name == &p.gameprofile.name)
                    });
                    if let Some(player) = target {
                        player.send_system_message_raw(&component, overlay);
                    }
                }
            } else {
                for player in server.get_all_players() {
                    player.send_system_message_raw(&component, overlay);
                }
            }
            Ok(json!(true))
        }

        // ==========================================
        // Server Settings
        // ==========================================
        "minecraft:serversettings/autosave" => {
            check_no_params(params)?;
            Ok(json!(
                server
                    .management_hub
                    .settings
                    .autosave
                    .load(Ordering::Relaxed)
            ))
        }
        "minecraft:serversettings/autosave/set" => {
            let enable: bool = parse_param(params, "enable")?;
            server
                .management_hub
                .settings
                .autosave
                .store(enable, Ordering::Relaxed);
            Ok(json!(enable))
        }

        "minecraft:serversettings/difficulty" => {
            check_no_params(params)?;
            Ok(json!(
                server.management_hub.settings.difficulty.load().as_str()
            ))
        }
        "minecraft:serversettings/difficulty/set" => {
            let diff: String = parse_param(params, "difficulty")?;
            let lower = diff.to_lowercase();
            if matches!(lower.as_str(), "peaceful" | "easy" | "normal" | "hard") {
                server
                    .management_hub
                    .settings
                    .difficulty
                    .store(Arc::new(lower.clone()));
                Ok(json!(lower))
            } else {
                Err(RpcError::invalid_params(Some(format!(
                    "Invalid difficulty: {diff}"
                ))))
            }
        }

        "minecraft:serversettings/enforce_allowlist" => {
            check_no_params(params)?;
            Ok(json!(
                server
                    .management_hub
                    .settings
                    .enforce_allowlist
                    .load(Ordering::Relaxed)
            ))
        }
        "minecraft:serversettings/enforce_allowlist/set" => {
            let enforce: bool = parse_param(params, "enforce")?;
            server
                .management_hub
                .settings
                .enforce_allowlist
                .store(enforce, Ordering::Relaxed);
            if enforce {
                kick_non_whitelisted_players(server);
            }
            Ok(json!(enforce))
        }

        "minecraft:serversettings/use_allowlist" => {
            check_no_params(params)?;
            Ok(json!(server.white_list.load(Ordering::Relaxed)))
        }
        "minecraft:serversettings/use_allowlist/set" => {
            let use_list: bool = parse_param(params, "use")?;
            server.white_list.store(use_list, Ordering::Relaxed);
            if use_list
                && server
                    .management_hub
                    .settings
                    .enforce_allowlist
                    .load(Ordering::Relaxed)
            {
                kick_non_whitelisted_players(server);
            }
            Ok(json!(use_list))
        }

        "minecraft:serversettings/max_players" => {
            check_no_params(params)?;
            Ok(json!(
                server
                    .management_hub
                    .settings
                    .max_players
                    .load(Ordering::Relaxed)
            ))
        }
        "minecraft:serversettings/max_players/set" => {
            let max: u32 = parse_param(params, "max")?;
            server
                .management_hub
                .settings
                .max_players
                .store(max, Ordering::Relaxed);
            Ok(json!(max))
        }

        "minecraft:serversettings/pause_when_empty_seconds" => {
            check_no_params(params)?;
            Ok(json!(
                server
                    .management_hub
                    .settings
                    .pause_when_empty_seconds
                    .load(Ordering::Relaxed)
            ))
        }
        "minecraft:serversettings/pause_when_empty_seconds/set" => {
            let sec: i32 = parse_param(params, "seconds")?;
            server
                .management_hub
                .settings
                .pause_when_empty_seconds
                .store(sec, Ordering::Relaxed);
            Ok(json!(sec))
        }

        "minecraft:serversettings/player_idle_timeout" => {
            check_no_params(params)?;
            Ok(json!(server.player_idle_timeout.load(Ordering::Relaxed)))
        }
        "minecraft:serversettings/player_idle_timeout/set" => {
            let sec: i32 = parse_param(params, "seconds")?;
            server.player_idle_timeout.store(sec, Ordering::Relaxed);
            Ok(json!(sec))
        }

        "minecraft:serversettings/allow_flight" => {
            check_no_params(params)?;
            Ok(json!(
                server
                    .management_hub
                    .settings
                    .allow_flight
                    .load(Ordering::Relaxed)
            ))
        }
        "minecraft:serversettings/allow_flight/set" => {
            let allow: bool = parse_param(params, "allow")?;
            server
                .management_hub
                .settings
                .allow_flight
                .store(allow, Ordering::Relaxed);
            Ok(json!(allow))
        }

        "minecraft:serversettings/motd" => {
            check_no_params(params)?;
            Ok(json!(server.management_hub.settings.motd.load().as_str()))
        }
        "minecraft:serversettings/motd/set" => {
            let motd: String = parse_param(params, "message")?;
            server
                .management_hub
                .settings
                .motd
                .store(Arc::new(motd.clone()));
            Ok(json!(motd))
        }

        "minecraft:serversettings/spawn_protection_radius" => {
            check_no_params(params)?;
            Ok(json!(
                server
                    .management_hub
                    .settings
                    .spawn_protection_radius
                    .load(Ordering::Relaxed)
            ))
        }
        "minecraft:serversettings/spawn_protection_radius/set" => {
            let radius: u32 = parse_param(params, "radius")?;
            server
                .management_hub
                .settings
                .spawn_protection_radius
                .store(radius, Ordering::Relaxed);
            Ok(json!(radius))
        }

        "minecraft:serversettings/force_game_mode" => {
            check_no_params(params)?;
            Ok(json!(
                server
                    .management_hub
                    .settings
                    .force_game_mode
                    .load(Ordering::Relaxed)
            ))
        }
        "minecraft:serversettings/force_game_mode/set" => {
            let force: bool = parse_param(params, "force")?;
            server
                .management_hub
                .settings
                .force_game_mode
                .store(force, Ordering::Relaxed);
            Ok(json!(force))
        }

        "minecraft:serversettings/game_mode" => {
            check_no_params(params)?;
            Ok(json!(
                server.management_hub.settings.game_mode.load().as_str()
            ))
        }
        "minecraft:serversettings/game_mode/set" => {
            let mode: String = parse_param(params, "mode")?;
            let lower = mode.to_lowercase();
            if matches!(
                lower.as_str(),
                "survival" | "creative" | "adventure" | "spectator"
            ) {
                server
                    .management_hub
                    .settings
                    .game_mode
                    .store(Arc::new(lower.clone()));
                Ok(json!(lower))
            } else {
                Err(RpcError::invalid_params(Some(format!(
                    "Invalid game mode: {mode}"
                ))))
            }
        }

        "minecraft:serversettings/view_distance" => {
            check_no_params(params)?;
            Ok(json!(
                server
                    .management_hub
                    .settings
                    .view_distance
                    .load(Ordering::Relaxed)
            ))
        }
        "minecraft:serversettings/view_distance/set" => {
            let dist: u32 = parse_param(params, "distance")?;
            server
                .management_hub
                .settings
                .view_distance
                .store(dist, Ordering::Relaxed);
            Ok(json!(dist))
        }

        "minecraft:serversettings/simulation_distance" => {
            check_no_params(params)?;
            Ok(json!(
                server
                    .management_hub
                    .settings
                    .simulation_distance
                    .load(Ordering::Relaxed)
            ))
        }
        "minecraft:serversettings/simulation_distance/set" => {
            let dist: u32 = parse_param(params, "distance")?;
            server
                .management_hub
                .settings
                .simulation_distance
                .store(dist, Ordering::Relaxed);
            Ok(json!(dist))
        }

        "minecraft:serversettings/accept_transfers" => {
            check_no_params(params)?;
            Ok(json!(
                server
                    .management_hub
                    .settings
                    .accept_transfers
                    .load(Ordering::Relaxed)
            ))
        }
        "minecraft:serversettings/accept_transfers/set" => {
            let accept: bool = parse_param(params, "accept")?;
            server
                .management_hub
                .settings
                .accept_transfers
                .store(accept, Ordering::Relaxed);
            Ok(json!(accept))
        }

        "minecraft:serversettings/status_heartbeat_interval" => {
            check_no_params(params)?;
            Ok(json!(
                server
                    .management_hub
                    .settings
                    .status_heartbeat_interval
                    .load(Ordering::Relaxed)
            ))
        }
        "minecraft:serversettings/status_heartbeat_interval/set" => {
            let sec: u64 = parse_param(params, "seconds")?;
            server
                .management_hub
                .settings
                .status_heartbeat_interval
                .store(sec, Ordering::Relaxed);
            Ok(json!(sec))
        }

        "minecraft:serversettings/operator_user_permission_level" => {
            check_no_params(params)?;
            Ok(json!(
                server
                    .management_hub
                    .settings
                    .operator_user_permission_level
                    .load(Ordering::Relaxed)
            ))
        }
        "minecraft:serversettings/operator_user_permission_level/set" => {
            let lvl: u8 = parse_param(params, "level")?;
            if lvl <= 4 {
                server
                    .management_hub
                    .settings
                    .operator_user_permission_level
                    .store(lvl, Ordering::Relaxed);
                Ok(json!(lvl))
            } else {
                Err(RpcError::invalid_params(Some(
                    "Permission level must be between 0 and 4".to_string(),
                )))
            }
        }

        "minecraft:serversettings/hide_online_players" => {
            check_no_params(params)?;
            Ok(json!(
                server
                    .management_hub
                    .settings
                    .hide_online_players
                    .load(Ordering::Relaxed)
            ))
        }
        "minecraft:serversettings/hide_online_players/set" => {
            let hide: bool = parse_param(params, "hide")?;
            server
                .management_hub
                .settings
                .hide_online_players
                .store(hide, Ordering::Relaxed);
            Ok(json!(hide))
        }

        "minecraft:serversettings/status_replies" => {
            check_no_params(params)?;
            Ok(json!(
                server
                    .management_hub
                    .settings
                    .status_replies
                    .load(Ordering::Relaxed)
            ))
        }
        "minecraft:serversettings/status_replies/set" => {
            let enable: bool = parse_param(params, "enable")?;
            server
                .management_hub
                .settings
                .status_replies
                .store(enable, Ordering::Relaxed);
            Ok(json!(enable))
        }

        "minecraft:serversettings/entity_broadcast_range" => {
            check_no_params(params)?;
            Ok(json!(
                server
                    .management_hub
                    .settings
                    .entity_broadcast_range
                    .load(Ordering::Relaxed)
            ))
        }
        "minecraft:serversettings/entity_broadcast_range/set" => {
            let pts: u32 = parse_param(params, "percentage_points")?;
            server
                .management_hub
                .settings
                .entity_broadcast_range
                .store(pts, Ordering::Relaxed);
            Ok(json!(pts))
        }

        // ==========================================
        // Gamerules
        // ==========================================
        "minecraft:gamerules" => {
            check_no_params(params)?;
            let rules = get_gamerules(server);
            Ok(serde_json::to_value(rules).unwrap_or(Value::Null))
        }
        "minecraft:gamerules/update" => {
            let update: UntypedGameRule = parse_param(params, "gamerule")?;
            let Some(rule) = GameRule::all().iter().find(|r| r.to_string() == update.key) else {
                return Err(RpcError::invalid_params(Some(format!(
                    "Unknown gamerule: {}",
                    update.key
                ))));
            };

            let typed = update_gamerule(server, rule, &update.value)?;
            server.management_hub.broadcast_gamerule_changed(&typed);
            Ok(serde_json::to_value(typed).unwrap_or(Value::Null))
        }

        unknown => Err(RpcError::method_not_found(Some(format!(
            "Method not found: {unknown}"
        )))),
    }
}

fn get_allowlist(server: &Server) -> Vec<PlayerDto> {
    let guard = server
        .data
        .whitelist_config
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    guard
        .whitelist
        .iter()
        .map(|e| PlayerDto::new(Some(e.uuid), Some(e.name.clone())))
        .collect()
}

fn get_bans(server: &Server) -> Vec<UserBanDto> {
    let guard = server
        .data
        .banned_player_list
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    guard
        .banned_players
        .iter()
        .map(|b| UserBanDto {
            player: PlayerDto::new(Some(b.uuid), Some(b.name.clone())),
            reason: if b.reason.is_empty() {
                None
            } else {
                Some(b.reason.clone())
            },
            source: if b.source.is_empty() {
                None
            } else {
                Some(b.source.clone())
            },
            expires: b.expires.and_then(|exp| {
                exp.format(&time::format_description::well_known::Rfc3339)
                    .ok()
            }),
        })
        .collect()
}

fn get_ip_bans(server: &Server) -> Vec<IpBanDto> {
    let guard = server
        .data
        .banned_ip_list
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    guard
        .banned_ips
        .iter()
        .map(|b| IpBanDto {
            ip: Some(b.ip.to_string()),
            reason: if b.reason.is_empty() {
                None
            } else {
                Some(b.reason.clone())
            },
            source: if b.source.is_empty() {
                None
            } else {
                Some(b.source.clone())
            },
            expires: b.expires.and_then(|exp| {
                exp.format(&time::format_description::well_known::Rfc3339)
                    .ok()
            }),
        })
        .collect()
}

fn get_operators(server: &Server) -> Vec<OperatorDto> {
    let guard = server
        .data
        .operator_config
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    guard
        .ops
        .iter()
        .map(|o| OperatorDto {
            player: PlayerDto::new(Some(o.uuid), Some(o.name.clone())),
            permission_level: Some(o.level as u8),
            bypasses_player_limit: Some(o.bypasses_player_limit),
        })
        .collect()
}

fn get_gamerules(server: &Server) -> Vec<TypedGameRule> {
    let level_data = server.level_info.load();
    let mut rules = Vec::new();
    for rule in GameRule::all() {
        match level_data.game_rules.get(rule) {
            GameRuleValue::Int(v) => {
                rules.push(TypedGameRule {
                    key: rule.to_string(),
                    type_name: "integer".to_string(),
                    value: json!(v),
                });
            }
            GameRuleValue::Bool(v) => {
                rules.push(TypedGameRule {
                    key: rule.to_string(),
                    type_name: "boolean".to_string(),
                    value: json!(v),
                });
            }
        }
    }
    rules
}

fn update_gamerule(
    server: &Server,
    rule: &GameRule,
    value: &Value,
) -> Result<TypedGameRule, RpcError> {
    let level_data = server.level_info.load();
    match level_data.game_rules.get(rule) {
        GameRuleValue::Int(_) => {
            let int_val = value
                .as_i64()
                .or_else(|| value.as_str().and_then(|s| s.parse::<i64>().ok()))
                .ok_or_else(|| {
                    RpcError::invalid_params(Some(format!(
                        "Stated type mismatches with actual type of gamerule \"{rule}\""
                    )))
                })?;
            for world in server.worlds.load().iter() {
                world.set_game_rule(rule, GameRuleValue::Int(int_val));
            }
            Ok(TypedGameRule {
                key: rule.to_string(),
                type_name: "integer".to_string(),
                value: json!(int_val),
            })
        }
        GameRuleValue::Bool(_) => {
            let bool_val = value
                .as_bool()
                .or_else(|| value.as_str().and_then(|s| bool::from_str(s).ok()))
                .ok_or_else(|| {
                    RpcError::invalid_params(Some(format!(
                        "Stated type mismatches with actual type of gamerule \"{rule}\""
                    )))
                })?;
            for world in server.worlds.load().iter() {
                world.set_game_rule(rule, GameRuleValue::Bool(bool_val));
            }
            Ok(TypedGameRule {
                key: rule.to_string(),
                type_name: "boolean".to_string(),
                value: json!(bool_val),
            })
        }
    }
}
