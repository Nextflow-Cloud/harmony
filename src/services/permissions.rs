use serde::{Deserialize, Serialize};

use super::database::{
    channels::Channel,
    members::Member,
    roles::{get_role, Role},
    spaces::get_space,
};

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[repr(i64)]
pub enum Permission {
    Administrator = 0x1,    // 1 << 0
    ViewChannel = 0x2,      // 1 << 1
    SendMessages = 0x4,     // 1 << 2
    DeleteMessages = 0x8,   // 1 << 3
    CreateInvite = 0x10,    // 1 << 4
    SuspendMembers = 0x20,  // 1 << 5
    KickMembers = 0x40,     // 1 << 6
    BanMembers = 0x80,      // 1 << 7
    ManageChannels = 0x100, // 1 << 8
    ManageInvites = 0x200,  // 1 << 9
    ManageRoles = 0x400,    // 1 << 10
    ManageSpace = 0x800,    // 1 << 11
}

#[derive(Clone, Debug)]
pub struct PermissionSet {
    permissions: i64,
}

impl Permission {
    pub fn iter() -> impl Iterator<Item = Self> {
        [
            Permission::Administrator,
            Permission::ViewChannel,
            Permission::SendMessages,
            Permission::DeleteMessages,
            Permission::CreateInvite,
            Permission::SuspendMembers,
            Permission::KickMembers,
            Permission::BanMembers,
            Permission::ManageChannels,
            Permission::ManageInvites,
            Permission::ManageRoles,
            Permission::ManageSpace,
        ]
        .iter()
        .copied()
    }
}

impl Serialize for PermissionSet {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_i64(self.permissions)
    }
}

impl<'de> Deserialize<'de> for PermissionSet {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let permissions = i64::deserialize(deserializer)?;
        Ok(PermissionSet { permissions })
    }
}

impl PermissionSet {
    pub fn new() -> Self {
        Self { permissions: 0 }
    }

    pub fn all() -> Self {
        Self {
            permissions: i64::MAX,
        }
    }

    pub fn to_i64(&self) -> i64 {
        self.permissions
    }

    pub fn to_vec(&self) -> Vec<Permission> {
        let mut permissions = Vec::new();
        for permission in Permission::iter() {
            if self.has_permission(permission) {
                permissions.push(permission);
            }
        }
        permissions
    }

    pub fn has_permission(&self, permission: Permission) -> bool {
        self.permissions & permission as i64 != 0
    }

    pub fn add_permission(&mut self, permission: Permission) {
        self.permissions |= permission as i64;
    }

    pub fn remove_permission(&mut self, permission: Permission) {
        self.permissions &= !(permission as i64);
    }

    pub fn combine(&mut self, other: PermissionSet) {
        self.permissions |= other.permissions;
    }
}

impl From<i64> for PermissionSet {
    fn from(permissions: i64) -> Self {
        Self { permissions }
    }
}

pub async fn permissions_for(member: Member) -> PermissionSet {
    let space = get_space(member.space_id)
        .await
        .expect("Unexpected error: failed to get space");
    if space.owner == member.id {
        PermissionSet::all()
    } else {
        let mut calculated_permissions = PermissionSet::from(space.base_permissions);
        let roles_sorted = member.roles.clone();
        let futures = roles_sorted.iter().map(|role| get_role(role.to_string()));
        let mut roles = futures_util::future::try_join_all(futures)
            .await
            .expect("Unexpected error: failed to get roles");
        roles.sort_by(|a, b| a.position.cmp(&b.position));
        roles.reverse();
        let default = calculated_permissions.to_vec();
        for role in roles {
            let role_permissions: PermissionSet = role.permissions.into();
            if role_permissions.has_permission(Permission::Administrator) {
                calculated_permissions = PermissionSet::all();
                break;
            }
            for permission in &default {
                if !role_permissions.has_permission(permission.clone()) {
                    calculated_permissions.remove_permission(permission.clone());
                }
            }
            calculated_permissions.combine(role_permissions);
        }
        calculated_permissions
    }
}

pub async fn channel_permissions_for(member: Member, channel: Channel) {}

pub async fn is_owner(member: Member) -> bool {
    let space = get_space(member.space_id)
        .await
        .expect("Unexpected error: failed to get space");
    space.owner == member.id
}

pub async fn can_modify_role(member: Member, role: Role) -> bool {
    let space = get_space(member.space_id.clone())
        .await
        .expect("Unexpected error: failed to get space");
    if space.owner == member.id {
        return true;
    }
    let member_roles = member.roles.clone();
    let futures = member_roles.iter().map(|role| get_role(role.to_string()));
    let mut roles = futures_util::future::try_join_all(futures)
        .await
        .expect("Unexpected error: failed to get roles");
    roles.sort_by(|a, b| a.position.cmp(&b.position));
    roles.reverse();
    let permissions = permissions_for(member).await;
    if !permissions.has_permission(Permission::ManageRoles) {
        return false;
    }
    if role.position < roles[0].position {
        return true;
    }
    false
}

pub async fn has_permission(member: Member, permission: Permission) -> bool {
    let permissions = permissions_for(member).await;
    permissions.has_permission(permission)
}

// #[proc_macro_attribute]
// pub fn required_permission(attr: TokenStream, item: TokenStream) -> TokenStream {
//     let attr = parse_macro_input!(attr as AttributeArgs);
//     let item = parse_macro_input!(item as ItemFn);
//     let mut permissions = Vec::new();
//     for arg in attr {
//         if let NestedMeta::Meta(Meta::Path(path)) = arg {
//             let ident = path.get_ident().expect("Unexpected error: failed to get ident");
//             let permission = match ident.to_string().as_str() {
//                 "Administrator" => Permission::Administrator,
//                 "ViewChannel" => Permission::ViewChannel,
//                 "SendMessages" => Permission::SendMessages,
//                 "DeleteMessages" => Permission::DeleteMessages,
//                 "CreateInvite" => Permission::CreateInvite,
//                 "SuspendMembers" => Permission::SuspendMembers,
//                 "KickMembers" => Permission::KickMembers,
//                 "BanMembers" => Permission::BanMembers,
//                 "ManageChannels" => Permission::ManageChannels,
//                 "ManageInvites" => Permission::ManageInvites,
//                 "ManageRoles" => Permission::ManageRoles,
//                 "ManageSpace" => Permission::ManageSpace,
//                 _ => panic!("Unexpected error: invalid permission"),
//             };
//             permissions.push(permission);
//         }
//     }
//     // Append a permission check to the original function

// }
