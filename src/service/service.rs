pub struct ServiceRegistry;
impl ServiceRegistry {
    pub fn new() -> Self {
        Self
    }
}
impl Default for ServiceRegistry {
    fn default() -> Self {
        Self::new()
    }
}
