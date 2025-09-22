/// Access control (RBAC) placeholder
pub type Role = String;
pub fn has_role(_user_id: &str, _role: &Role) -> bool {
    true
}
