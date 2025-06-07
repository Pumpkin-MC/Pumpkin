use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Describes the default behavior for permissions
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PermissionDefault {
    /// Permission is not granted by default
    Deny,
    /// Permission is granted by default
    Allow,
    /// Permission is granted by default to operators
    Op(PermissionLvl),
}

/// Defines a permission node in the system
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Permission {
    /// The full node name (e.g., "minecraft:command.gamemode")
    pub node: String,
    /// Description of what this permission does
    pub description: String,
    /// The default value of this permission
    pub default: PermissionDefault,
    /// Children nodes that are affected by this permission
    pub children: HashMap<String, bool>,
}

impl Permission {
    pub fn new(node: &str, description: &str, default: PermissionDefault) -> Self {
        Self {
            node: node.to_string(),
            description: description.to_string(),
            default,
            children: HashMap::new(),
        }
    }

    /// Add a child permission with a specific value
    pub fn add_child(&mut self, child: &str, value: bool) -> &mut Self {
        self.children.insert(child.to_string(), value);
        self
    }
}

/// Repository for all registered permissions in the server
#[derive(Default)]
pub struct PermissionRegistry {
    /// All registered permissions
    permissions: HashMap<String, Permission>,
}

impl PermissionRegistry {
    pub fn new() -> Self {
        Self {
            permissions: HashMap::new(),
        }
    }

    /// Register a new permission
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

    /// Get a registered permission by node
    pub fn get_permission(&self, node: &str) -> Option<&Permission> {
        self.permissions.get(node)
    }

    /// Check if a permission is registered
    pub fn has_permission(&self, node: &str) -> bool {
        self.permissions.contains_key(node)
    }
}

/// Storage for player permissions
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct PermissionAttachment {
    /// Directly assigned permissions
    permissions: HashMap<String, bool>,
}

impl PermissionAttachment {
    pub fn new() -> Self {
        Self {
            permissions: HashMap::new(),
        }
    }

    /// Set a permission value
    pub fn set_permission(&mut self, node: &str, value: bool) {
        self.permissions.insert(node.to_string(), value);
    }

    /// Unset a permission
    pub fn unset_permission(&mut self, node: &str) {
        self.permissions.remove(node);
    }

    /// Check if a permission is directly set
    pub fn has_permission_set(&self, node: &str) -> Option<bool> {
        self.permissions.get(node).copied()
    }

    /// Get all directly set permissions
    pub fn get_permissions(&self) -> &HashMap<String, bool> {
        &self.permissions
    }
}

/// Manager for player permissions
#[derive(Default)]
pub struct PermissionManager {
    /// Global registry of permissions
    pub registry: Arc<RwLock<PermissionRegistry>>,
    /// Player permission attachments
    pub attachments: HashMap<uuid::Uuid, Arc<RwLock<PermissionAttachment>>>,
}

impl PermissionManager {
    pub fn new(registry: Arc<RwLock<PermissionRegistry>>) -> Self {
        Self {
            registry,
            attachments: HashMap::new(),
        }
    }

    /// Get or create a player's permission attachment
    pub fn get_attachment(&mut self, player_id: uuid::Uuid) -> Arc<RwLock<PermissionAttachment>> {
        self.attachments
            .entry(player_id)
            .or_insert_with(|| Arc::new(RwLock::new(PermissionAttachment::new())))
            .clone()
    }

    /// Remove a player's permission attachment
    pub fn remove_attachment(&mut self, player_id: &uuid::Uuid) {
        self.attachments.remove(player_id);
    }

    /// Check if a player has a permission, considering defaults and op status
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

            // Check for exact permission match
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
                if let Some(value) = attachment.has_permission_set(&format!("{}:*", current_node)) {
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
                            attachment.has_permission_set(&format!("{}.*", current_node))
                        {
                            return value;
                        }
                        current_node.push('.');
                    }
                }
            }

            // Check for inherited permissions from parent nodes
            for (node, value) in attachment.get_permissions() {
                if let Some(permission) = reg.get_permission(node) {
                    if permission.children.contains_key(permission_node) {
                        return *value && *permission.children.get(permission_node).unwrap();
                    }
                }
            }
        }

        // Fall back to default permission value
        if let Some(permission) = reg.get_permission(permission_node) {
            match permission.default {
                PermissionDefault::Allow => true,
                PermissionDefault::Deny => false,
                PermissionDefault::Op(required_level) => player_op_level >= required_level,
            }
        } else {
            // If permission isn't registered, default to deny
            false
        }
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
    #[default]
    Zero = 0,
    One = 1,
    Two = 2,
    Three = 3,
    Four = 4,
}

impl PartialOrd for PermissionLvl {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some((*self as u8).cmp(&(*other as u8)))
    }
}

impl Ord for PermissionLvl {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (*self as u8).cmp(&(*other as u8))
    }
}

impl Serialize for PermissionLvl {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u8(*self as u8)
    }
}

impl<'de> Deserialize<'de> for PermissionLvl {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = u8::deserialize(deserializer)?;
        match value {
            0 => Ok(PermissionLvl::Zero),
            2 => Ok(PermissionLvl::Two),
            3 => Ok(PermissionLvl::Three),
            4 => Ok(PermissionLvl::Four),
            _ => Err(serde::de::Error::custom(format!(
                "Invalid value for OpLevel: {value}"
            ))),
        }
    }
}
