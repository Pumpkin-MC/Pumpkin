use dashmap::DashMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;

/// Describes the default behaviour for permissions.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PermissionDefault {
    /// Permission is not granted by default.
    Deny,
    /// Permission is granted by default.
    Allow,
    /// Permission is granted by default to operators.
    Op(PermissionLvl),
}

/// Defines a permission node in the system.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Permission {
    /// The full node name (e.g., "minecraft:command.gamemode").
    pub node: String,
    /// Description of what this permission does.
    pub description: String,
    /// The default value of this permission.
    pub default: PermissionDefault,
    /// Children nodes that are affected by this permission.
    pub children: HashMap<String, bool>,
}

impl Permission {
    /// Creates a new `Permission` instance.
    ///
    /// # Parameters
    /// - `node`: The full permission node string (e.g., `"minecraft:command.gamemode"`).
    /// - `description`: A human-readable description of what this permission does.
    /// - `default`: The default behaviour of the permission (`PermissionDefault`).
    ///
    /// # Returns
    /// A new `Permission` with an empty set of children.
    #[must_use]
    pub fn new(node: &str, description: &str, default: PermissionDefault) -> Self {
        Self {
            node: node.to_string(),
            description: description.to_string(),
            default,
            children: HashMap::new(),
        }
    }

    /// Adds a child permission node to this permission.
    ///
    /// # Arguments
    /// * `child` - Child node name.
    /// * `value` - Whether the child is allowed by default.
    ///
    /// # Returns
    /// Mutable reference to self for chaining.
    pub fn add_child(&mut self, child: &str, value: bool) -> &mut Self {
        self.children.insert(child.to_string(), value);
        self
    }
}

/// Repository for all registered permissions in the server.
#[derive(Default)]
pub struct PermissionRegistry {
    /// All registered permissions.
    permissions: DashMap<String, Permission>,
}

impl PermissionRegistry {
    /// Creates a new empty `PermissionRegistry`.
    ///
    /// # Returns
    /// A `PermissionRegistry` with no permissions registered.
    #[must_use]
    pub fn new() -> Self {
        Self {
            permissions: DashMap::new(),
        }
    }

    /// Registers a new permission in the registry.
    ///
    /// # Parameters
    /// - `permission`: The `Permission` instance to add.
    ///
    /// # Returns
    /// - `Ok(())` if the permission was successfully registered.
    /// - `Err(String)` if a permission with the same node already exists.
    pub fn register_permission(&self, permission: Permission) -> Result<(), String> {
        if self.permissions.contains_key(&permission.node) {
            return Err(format!(
                "Permission {} is already registered",
                permission.node
            ));
        }
        self.permissions.insert(permission.node.clone(), permission);
        Ok(())
    }

    /// Registers a new permission in the registry and expects it to be registered.
    ///
    /// # Panics
    ///
    /// Panics if the permission could not be registered (if one with the same node already exists).
    ///
    /// # Parameters
    /// - `permission`: The `Permission` instance to add.
    #[allow(clippy::expect_used)]
    pub fn register_permission_or_panic(&self, permission: Permission) {
        self.register_permission(permission)
            .expect("Permission should have been registered successfully");
    }

    /// Retrieves a permission node by its name.
    ///
    /// # Parameters
    /// - `node`: The full permission node string to look up.
    ///
    /// # Returns
    /// `Some(Ref<'_, String, Permission>)` if the node exists, or `None` otherwise.
    #[must_use]
    pub fn get_permission(
        &self,
        node: &str,
    ) -> Option<dashmap::mapref::one::Ref<'_, String, Permission>> {
        self.permissions.get(node)
    }

    /// Overrides the default behaviour of an already-registered permission.
    ///
    /// Used to apply per-command permission overrides from the server
    /// configuration after the built-in permissions have been registered.
    ///
    /// # Parameters
    /// - `node`: The full permission node string to update.
    /// - `default`: The new default behaviour to apply.
    ///
    /// # Returns
    /// - `true` if the node existed and its default was updated.
    /// - `false` if no permission with that node is registered.
    #[must_use]
    pub fn set_default(&self, node: &str, default: PermissionDefault) -> bool {
        if let Some(mut permission) = self.permissions.get_mut(node) {
            permission.default = default;
            true
        } else {
            false
        }
    }

    /// Checks whether a permission node exists in the registry.
    ///
    /// # Parameters
    /// - `node`: The permission node string to check.
    ///
    /// # Returns
    /// `true` if the node exists, `false` otherwise.
    #[must_use]
    pub fn has_permission(&self, node: &str) -> bool {
        self.permissions.contains_key(node)
    }
}

/// Storage for player permissions.
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct PermissionAttachment {
    /// Directly assigned permissions.
    pub permissions: HashMap<String, bool>,
}

impl PermissionAttachment {
    /// Creates a new empty `PermissionAttachment`.
    ///
    /// # Returns
    /// A `PermissionAttachment` with no permissions set.
    #[must_use]
    pub fn new() -> Self {
        Self {
            permissions: HashMap::new(),
        }
    }

    /// Sets a permission value for a specific node.
    ///
    /// # Parameters
    /// - `node`: The permission node string.
    /// - `value`: Whether the permission is granted (`true`) or denied (`false`).
    pub fn set_permission(&mut self, node: &str, value: bool) {
        self.permissions.insert(node.to_string(), value);
    }

    /// Removes a permission from this attachment.
    ///
    /// # Parameters
    /// - `node`: The permission node string to remove.
    pub fn unset_permission(&mut self, node: &str) {
        self.permissions.remove(node);
    }

    /// Checks if a permission is explicitly set.
    ///
    /// # Parameters
    /// - `node`: The permission node string to query.
    ///
    /// # Returns
    /// `Some(true)` if granted, `Some(false)` if denied, or `None` if not set.
    #[must_use]
    pub fn has_permission_set(&self, node: &str) -> Option<bool> {
        self.permissions.get(node).copied()
    }

    /// Returns a reference to all set permissions.
    ///
    /// # Returns
    /// A `&HashMap<String, bool>` containing all permission nodes and their values.
    #[must_use]
    pub const fn get_permissions(&self) -> &HashMap<String, bool> {
        &self.permissions
    }
}

/// Manager for player and server permissions.
#[derive(Default)]
pub struct PermissionManager {
    /// Global registry of permissions.
    pub registry: PermissionRegistry,
    /// Player permission attachments keyed by player UUID.
    pub attachments: DashMap<uuid::Uuid, DashMap<String, bool>>,
}

impl PermissionManager {
    /// Creates a new empty `PermissionManager`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            registry: PermissionRegistry::new(),
            attachments: DashMap::new(),
        }
    }

    /// Creates a new `PermissionManager` with an existing `PermissionRegistry`.
    #[must_use]
    pub fn with_registry(registry: PermissionRegistry) -> Self {
        Self {
            registry,
            attachments: DashMap::new(),
        }
    }

    /// Registers a new permission node in the global registry.
    pub fn register_permission(&self, permission: Permission) -> Result<(), String> {
        self.registry.register_permission(permission)
    }

    /// Registers a new permission node in the global registry or panics if already registered.
    pub fn register_permission_or_panic(&self, permission: Permission) {
        self.registry.register_permission_or_panic(permission);
    }

    /// Retrieves a permission node by its name from the registry.
    #[must_use]
    pub fn get_permission(
        &self,
        node: &str,
    ) -> Option<dashmap::mapref::one::Ref<'_, String, Permission>> {
        self.registry.get_permission(node)
    }

    /// Overrides the default behaviour of an already-registered permission in the registry.
    #[must_use]
    pub fn set_default(&self, node: &str, default: PermissionDefault) -> bool {
        self.registry.set_default(node, default)
    }

    /// Checks whether a permission node exists in the registry.
    #[must_use]
    pub fn has_registered_permission(&self, node: &str) -> bool {
        self.registry.has_permission(node)
    }

    /// Sets a player's explicit permission.
    pub fn set_permission(&self, player_id: uuid::Uuid, node: impl Into<String>, value: bool) {
        self.attachments
            .entry(player_id)
            .or_default()
            .insert(node.into(), value);
    }

    /// Unsets a player's explicit permission.
    pub fn unset_permission(&self, player_id: &uuid::Uuid, node: &str) {
        if let Some(player_perms) = self.attachments.get(player_id) {
            player_perms.remove(node);
        }
    }

    /// Checks if a permission is explicitly set for a player.
    #[must_use]
    pub fn has_permission_set(&self, player_id: &uuid::Uuid, node: &str) -> Option<bool> {
        self.attachments
            .get(player_id)
            .and_then(|p| p.get(node).as_deref().copied())
    }

    /// Removes all permission attachments for a player.
    pub fn remove_attachment(&self, player_id: &uuid::Uuid) {
        self.attachments.remove(player_id);
    }

    /// Retrieves all explicitly assigned permissions for a player.
    #[must_use]
    pub fn get_player_permissions(&self, player_id: &uuid::Uuid) -> Option<HashMap<String, bool>> {
        self.attachments.get(player_id).map(|p| {
            p.iter()
                .map(|item| (item.key().clone(), *item.value()))
                .collect()
        })
    }

    /// Checks if a player has a specific permission.
    ///
    /// # Parameters
    /// - `player_id`: The UUID of the player.
    /// - `permission_node`: The permission node string to check (e.g., "minecraft:command.gamemode").
    /// - `player_op_level`: The operator level of the player (`PermissionLvl`).
    ///
    /// # Returns
    /// `true` if the player has the permission, `false` otherwise.
    #[must_use]
    pub fn has_permission(
        &self,
        player_id: &uuid::Uuid,
        permission_node: &str,
        player_op_level: PermissionLvl,
    ) -> bool {
        // Check explicitly set permissions
        if let Some(attachment) = self.attachments.get(player_id) {
            // Check for the exact permission match
            if let Some(value) = attachment.get(permission_node) {
                return *value;
            }

            // Check parent nodes (for wildcard permissions)
            let node_parts: Vec<&str> = permission_node.split(':').collect();
            if node_parts.len() == 2 {
                let namespace = node_parts[0];
                let key_parts: Vec<&str> = node_parts[1].split('.').collect();

                // Check wildcard permissions at each level
                let mut current_node = namespace.to_string();
                if let Some(value) = attachment.get(&format!("{current_node}:*")) {
                    return *value;
                }

                current_node.push(':');
                for (i, part) in key_parts.iter().enumerate() {
                    current_node.push_str(part);

                    if let Some(value) = attachment.get(&current_node) {
                        return *value;
                    }

                    if i < key_parts.len() - 1 {
                        if let Some(value) = attachment.get(&format!("{current_node}.*")) {
                            return *value;
                        }
                        current_node.push('.');
                    }
                }
            }

            // Check for inherited permissions from parent nodes
            for item in attachment.iter() {
                let node = item.key();
                let value = *item.value();
                if let Some(permission) = self.registry.get_permission(node)
                    && let Some(child_val) = permission.children.get(permission_node)
                {
                    return value && *child_val;
                }
            }
        }

        // Fall back to the default permission value
        self.registry
            .get_permission(permission_node)
            .is_some_and(|permission| match permission.default {
                PermissionDefault::Allow => true,
                PermissionDefault::Deny => false,
                PermissionDefault::Op(required_level) => player_op_level >= required_level,
            })
    }
}

/// Represents the player's permission level
///
/// Permission levels determine the player's access to commands and server operations.
/// Each numeric level corresponds to a specific role:
/// - `Zero`: `normal`: Player can use basic commands.
/// - `One`: `moderator`: Player can bypass spawn protection.
/// - `Two`: `gamemaster`: Player or executor can use more commands and player can use command blocks.
/// - `Three`:  `admin`: Player or executor can use commands related to multiplayer management.
/// - `Four`: `owner`: Player or executor can use all of the commands, including commands related to server management.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PermissionLvl {
    /// Normal player. Can use basic commands.
    #[default]
    Zero = 0,
    /// Moderator. Can bypass spawn protection.
    One = 1,
    /// Gamemaster. Can use additional commands, including command blocks.
    Two = 2,
    /// Admin. Can manage multiplayer commands and moderate players.
    Three = 3,
    /// Owner. Full access to all commands and server management.
    Four = 4,
}

impl PartialOrd for PermissionLvl {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PermissionLvl {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (*self as u8).cmp(&(*other as u8))
    }
}

impl Serialize for PermissionLvl {
    fn serialize<S: Serializer>(
        &self,
        serializer: S,
    ) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error> {
        serializer.serialize_u8(*self as u8)
    }
}

impl<'de> Deserialize<'de> for PermissionLvl {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = u8::deserialize(deserializer)?;
        match value {
            0 => Ok(Self::Zero),
            2 => Ok(Self::Two),
            3 => Ok(Self::Three),
            4 => Ok(Self::Four),
            _ => Err(serde::de::Error::custom(format!(
                "Invalid value for OpLevel: {value}"
            ))),
        }
    }
}
