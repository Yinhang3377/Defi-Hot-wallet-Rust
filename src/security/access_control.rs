// src/security/access_control.rs
use crate::tools::error::WalletError;
use std::collections::HashMap;

/// 瑙掕壊瀹氫箟
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

/// 鏉冮檺瀹氫箟
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

/// 璁块棶鎺у埗绠＄悊鍣?pub struct AccessControl {
    user_roles: HashMap<String, Vec<Role>>,
    role_permissions: HashMap<Role, Vec<Permission>>,
}

impl AccessControl {
    /// 鍒涘缓鏂扮殑璁块棶鎺у埗绠＄悊鍣?    pub fn new() -> Self {
        let mut role_permissions = HashMap::new();

        // 瀹氫箟瑙掕壊鏉冮檺
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

    /// 涓虹敤鎴峰垎閰嶈鑹?    pub fn assign_role(&mut self, user_id: &str, role: Role) -> Result<(), WalletError> {
        self.user_roles.entry(user_id.to_string()).or_default().push(role);
        Ok(())
    }

    /// 鎾ら攢鐢ㄦ埛瑙掕壊
    pub fn revoke_role(&mut self, user_id: &str, role: &Role) -> Result<(), WalletError> {
        if let Some(roles) = self.user_roles.get_mut(user_id) {
            roles.retain(|r| r != role);
        }
        Ok(())
    }

    /// 妫€鏌ョ敤鎴锋槸鍚︽湁鎸囧畾瑙掕壊
    pub fn has_role(&self, user_id: &str, role: &Role) -> bool {
        self.user_roles.get(user_id).map(|roles| roles.contains(role)).unwrap_or(false)
    }

    /// 妫€鏌ョ敤鎴锋槸鍚︽湁鎸囧畾鏉冮檺
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

    /// 鑾峰彇鐢ㄦ埛鐨勬墍鏈夎鑹?    pub fn get_user_roles(&self, user_id: &str) -> Vec<Role> {
        self.user_roles.get(user_id).cloned().unwrap_or_default()
    }

    /// 鑾峰彇瑙掕壊鐨勬墍鏈夋潈闄?    pub fn get_role_permissions(&self, role: &Role) -> Vec<Permission> {
        self.role_permissions.get(role).cloned().unwrap_or_default()
    }

    /// 妫€鏌ョ敤鎴锋槸鍚︿负绠＄悊鍛?    pub fn is_admin(&self, user_id: &str) -> bool {
        self.has_role(user_id, &Role::Admin)
    }

    /// 妫€鏌ョ敤鎴锋槸鍚︿负瀹¤鍛?    pub fn is_auditor(&self, user_id: &str) -> bool {
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

        // 鍒嗛厤瑙掕壊
        ac.assign_role(user_id, Role::User).unwrap();
        assert!(ac.has_role(user_id, &Role::User));

        // 妫€鏌ユ潈闄?        assert!(ac.has_permission(user_id, &Permission::CreateWallet));
        assert!(ac.has_permission(user_id, &Permission::ViewBalance));
        assert!(!ac.has_permission(user_id, &Permission::AuditLogs));
    }

    #[test]
    fn test_role_revocation() {
        let mut ac = AccessControl::new();
        let user_id = "user123";

        // 鍒嗛厤骞舵挙閿€瑙掕壊
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

        // 瀹¤鍛樺簲璇ユ湁鏌ョ湅浣欓鍜屽璁℃棩蹇楃殑鏉冮檺
        assert!(ac.has_permission(user_id, &Permission::ViewBalance));
        assert!(ac.has_permission(user_id, &Permission::AuditLogs));

        // 浣嗕笉搴旇鏈夌鐞嗙敤鎴风殑鏉冮檺
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
