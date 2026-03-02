use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

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
    permissions: HashMap<String, Permission>,
}

impl PermissionRegistry {
    /// Creates a new empty `PermissionRegistry`.
    ///
    /// # Returns
    /// A `PermissionRegistry` with no permissions registered.
    #[must_use]
    pub fn new() -> Self {
        Self {
            permissions: HashMap::new(),
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
    pub fn register_permission(&mut self, permission: Permission) -> Result<(), String> {
        if self.permissions.contains_key(&permission.node) {
            return Err(format!(
                "Permission {} is already registered",
                permission.node
            ));
        }
        self.permissions.insert(permission.node.clone(), permission);
        Ok(())
    }

    /// Retrieves a permission node by its name.
    ///
    /// # Parameters
    /// - `node`: The full permission node string to look up.
    ///
    /// # Returns
    /// `Some(&Permission)` if the node exists, or `None` otherwise.
    #[must_use]
    pub fn get_permission(&self, node: &str) -> Option<&Permission> {
        self.permissions.get(node)
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
    permissions: HashMap<String, bool>,
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

/// Manager for player permissions.
#[derive(Default)]
pub struct PermissionManager {
    /// Global registry of permissions.
    pub registry: Arc<RwLock<PermissionRegistry>>,
    /// Player permission attachments.
    pub attachments: HashMap<uuid::Uuid, Arc<RwLock<PermissionAttachment>>>,
}

impl PermissionManager {
    /// Creates a new `PermissionManager`.
    ///
    /// # Parameters
    /// - `registry`: An `Arc<RwLock<PermissionRegistry>>` containing the global permissions registry.
    ///
    /// # Returns
    /// A `PermissionManager` with an empty attachments map.
    pub fn new(registry: Arc<RwLock<PermissionRegistry>>) -> Self {
        Self {
            registry,
            attachments: HashMap::new(),
        }
    }

    /// Retrieves the `PermissionAttachment` for a given player, creating one if it doesn't exist.
    ///
    /// # Parameters
    /// - `player_id`: The UUID of the player.
    ///
    /// # Returns
    /// An `Arc<RwLock<PermissionAttachment>>` representing the player's attachment.
    pub fn get_attachment(&mut self, player_id: uuid::Uuid) -> Arc<RwLock<PermissionAttachment>> {
        self.attachments
            .entry(player_id)
            .or_insert_with(|| Arc::new(RwLock::new(PermissionAttachment::new())))
            .clone()
    }

    /// Removes the `PermissionAttachment` for a given player.
    ///
    /// # Parameters
    /// - `player_id`: The UUID of the player.
    pub fn remove_attachment(&mut self, player_id: &uuid::Uuid) {
        self.attachments.remove(player_id);
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
    pub async fn has_permission(
        &self,
        player_id: &uuid::Uuid,
        permission_node: &str,
        player_op_level: PermissionLvl,
    ) -> bool {
        let reg = self.registry.read().await;

        // Check explicitly set permissions
        if let Some(attachment) = self.attachments.get(player_id) {
            let attachment = attachment.read().await;

            // Check for the exact permission match
            if let Some(value) = attachment.has_permission_set(permission_node) {
                return value;
            }

            // Check parent nodes (for wildcard permissions)
            let node_parts: Vec<&str> = permission_node.split(':').collect();
            if node_parts.len() == 2 {
                let namespace = node_parts[0];
                let key_parts: Vec<&str> = node_parts[1].split('.').collect();

                // Check wildcard permissions at each level
                let mut current_node = namespace.to_string();
                if let Some(value) = attachment.has_permission_set(&format!("{current_node}:*")) {
                    return value;
                }

                current_node.push(':');
                for (i, part) in key_parts.iter().enumerate() {
                    current_node.push_str(part);

                    if let Some(value) = attachment.has_permission_set(&current_node) {
                        return value;
                    }

                    if i < key_parts.len() - 1 {
                        if let Some(value) =
                            attachment.has_permission_set(&format!("{current_node}.*"))
                        {
                            return value;
                        }
                        current_node.push('.');
                    }
                }
            }

            // Check for inherited permissions from parent nodes
            for (node, value) in attachment.get_permissions() {
                if let Some(permission) = reg.get_permission(node)
                    && permission.children.contains_key(permission_node)
                {
                    return *value && *permission.children.get(permission_node).unwrap();
                }
            }
        }

        // Fall back to the default permission value
        reg.get_permission(permission_node)
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
