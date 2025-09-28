// src/security/access_control.rs
use crate::tools::error::WalletError;
use std::collections::HashMap;

/// 角色定义
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Role {
    Admin,
    User,
    Auditor,
    Guest,
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Role::Admin => write!(f, "admin"),
            Role::User => write!(f, "user"),
            Role::Auditor => write!(f, "auditor"),
            Role::Guest => write!(f, "guest"),
        }
    }
}

/// 权限定义
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Permission {
    CreateWallet,
    TransferFunds,
    ViewBalance,
    AuditLogs,
    ManageUsers,
    SystemConfig,
}

impl std::fmt::Display for Permission {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Permission::CreateWallet => write!(f, "create_wallet"),
            Permission::TransferFunds => write!(f, "transfer_funds"),
            Permission::ViewBalance => write!(f, "view_balance"),
            Permission::AuditLogs => write!(f, "audit_logs"),
            Permission::ManageUsers => write!(f, "manage_users"),
            Permission::SystemConfig => write!(f, "system_config"),
        }
    }
}

/// 访问控制管理器
pub struct AccessControl {
    user_roles: HashMap<String, Vec<Role>>,
    role_permissions: HashMap<Role, Vec<Permission>>,
}

impl AccessControl {
    /// 创建新的访问控制管理器
    pub fn new() -> Self {
        let mut role_permissions = HashMap::new();

        // 定义角色权限
        role_permissions.insert(
            Role::Admin,
            vec![
                Permission::CreateWallet,
                Permission::TransferFunds,
                Permission::ViewBalance,
                Permission::AuditLogs,
                Permission::ManageUsers,
                Permission::SystemConfig,
            ],
        );

        role_permissions.insert(
            Role::User,
            vec![Permission::CreateWallet, Permission::TransferFunds, Permission::ViewBalance],
        );

        role_permissions
            .insert(Role::Auditor, vec![Permission::ViewBalance, Permission::AuditLogs]);

        role_permissions.insert(Role::Guest, vec![Permission::ViewBalance]);

        Self { user_roles: HashMap::new(), role_permissions }
    }

    /// 为用户分配角色
    pub fn assign_role(&mut self, user_id: &str, role: Role) -> Result<(), WalletError> {
        self.user_roles.entry(user_id.to_string()).or_insert_with(Vec::new).push(role);
        Ok(())
    }

    /// 撤销用户角色
    pub fn revoke_role(&mut self, user_id: &str, role: &Role) -> Result<(), WalletError> {
        if let Some(roles) = self.user_roles.get_mut(user_id) {
            roles.retain(|r| r != role);
        }
        Ok(())
    }

    /// 检查用户是否有指定角色
    pub fn has_role(&self, user_id: &str, role: &Role) -> bool {
        self.user_roles.get(user_id).map(|roles| roles.contains(role)).unwrap_or(false)
    }

    /// 检查用户是否有指定权限
    pub fn has_permission(&self, user_id: &str, permission: &Permission) -> bool {
        if let Some(user_roles) = self.user_roles.get(user_id) {
            for role in user_roles {
                if let Some(role_permissions) = self.role_permissions.get(role) {
                    if role_permissions.contains(permission) {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// 获取用户的所有角色
    pub fn get_user_roles(&self, user_id: &str) -> Vec<Role> {
        self.user_roles.get(user_id).cloned().unwrap_or_default()
    }

    /// 获取角色的所有权限
    pub fn get_role_permissions(&self, role: &Role) -> Vec<Permission> {
        self.role_permissions.get(role).cloned().unwrap_or_default()
    }

    /// 检查用户是否为管理员
    pub fn is_admin(&self, user_id: &str) -> bool {
        self.has_role(user_id, &Role::Admin)
    }

    /// 检查用户是否为审计员
    pub fn is_auditor(&self, user_id: &str) -> bool {
        self.has_role(user_id, &Role::Auditor)
    }
}

impl Default for AccessControl {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_role_assignment() {
        let mut ac = AccessControl::new();
        let user_id = "user123";

        // 分配角色
        ac.assign_role(user_id, Role::User).unwrap();
        assert!(ac.has_role(user_id, &Role::User));

        // 检查权限
        assert!(ac.has_permission(user_id, &Permission::CreateWallet));
        assert!(ac.has_permission(user_id, &Permission::ViewBalance));
        assert!(!ac.has_permission(user_id, &Permission::AuditLogs));
    }

    #[test]
    fn test_role_revocation() {
        let mut ac = AccessControl::new();
        let user_id = "user123";

        // 分配并撤销角色
        ac.assign_role(user_id, Role::Admin).unwrap();
        assert!(ac.has_role(user_id, &Role::Admin));

        ac.revoke_role(user_id, &Role::Admin).unwrap();
        assert!(!ac.has_role(user_id, &Role::Admin));
    }

    #[test]
    fn test_permission_check() {
        let mut ac = AccessControl::new();
        let user_id = "user123";

        ac.assign_role(user_id, Role::Auditor).unwrap();

        // 审计员应该有查看余额和审计日志的权限
        assert!(ac.has_permission(user_id, &Permission::ViewBalance));
        assert!(ac.has_permission(user_id, &Permission::AuditLogs));

        // 但不应该有管理用户的权限
        assert!(!ac.has_permission(user_id, &Permission::ManageUsers));
    }

    #[test]
    fn test_admin_check() {
        let mut ac = AccessControl::new();
        let admin_id = "admin123";
        let user_id = "user123";

        ac.assign_role(admin_id, Role::Admin).unwrap();
        ac.assign_role(user_id, Role::User).unwrap();

        assert!(ac.is_admin(admin_id));
        assert!(!ac.is_admin(user_id));
    }
}
