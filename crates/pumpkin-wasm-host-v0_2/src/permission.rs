use crate::player::to_wit_permission_level;
use crate::pumpkin::{
    self,
    plugin::permission::{Permission, PermissionChild, PermissionDefault},
};
use pumpkin_wasm_host_common::state::PluginHostState;

impl pumpkin::plugin::permission::Host for PluginHostState {}

pub(crate) fn to_wit_permission(permission: pumpkin_util::permission::Permission) -> Permission {
    Permission {
        node: permission.node,
        description: permission.description,
        default: match permission.default {
            pumpkin_util::permission::PermissionDefault::Deny => PermissionDefault::Deny,
            pumpkin_util::permission::PermissionDefault::Allow => PermissionDefault::Allow,
            pumpkin_util::permission::PermissionDefault::Op(level) => {
                PermissionDefault::Op(to_wit_permission_level(level))
            }
        },
        children: permission
            .children
            .into_iter()
            .map(|(node, value)| PermissionChild { node, value })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pumpkin::plugin::permission::PermissionLevel;
    use pumpkin_util::PermissionLvl;
    use pumpkin_util::permission::{
        Permission as UtilPermission, PermissionDefault as UtilPermissionDefault,
    };

    #[test]
    fn registered_permission_keeps_its_default_and_children() {
        let mut fly = UtilPermission::new(
            "other:fly",
            "Lets a player fly",
            UtilPermissionDefault::Op(PermissionLvl::Three),
        );
        fly.add_child("other:fly.fast", false);

        let fly = to_wit_permission(fly);
        assert_eq!(fly.node, "other:fly");
        assert_eq!(fly.description, "Lets a player fly");
        assert!(matches!(
            fly.default,
            PermissionDefault::Op(PermissionLevel::Three)
        ));
        assert!(matches!(
            fly.children.as_slice(),
            [PermissionChild { node, value: false }] if node == "other:fly.fast"
        ));

        let allow = UtilPermission::new("other:home", "", UtilPermissionDefault::Allow);
        assert!(matches!(
            to_wit_permission(allow).default,
            PermissionDefault::Allow
        ));
        let deny = UtilPermission::new("other:vanish", "", UtilPermissionDefault::Deny);
        assert!(matches!(
            to_wit_permission(deny).default,
            PermissionDefault::Deny
        ));
    }
}
