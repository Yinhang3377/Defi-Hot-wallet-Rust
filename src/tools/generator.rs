//! generator.rs
//! 代码生成/脚手架工具模块

use rand::{ Rng, SeedableRng };
use rand::rngs::StdRng;
use std::sync::{ Arc, Mutex };
use std::collections::HashSet;

/// ID生成器，支持并发、种子输入、唯一性保证
pub struct IdGenerator {
    rng: Arc<Mutex<StdRng>>,
    generated_ids: Arc<Mutex<HashSet<u64>>>,
}

impl IdGenerator {
    pub fn new(seed: Option<u64>) -> Self {
        let rng = match seed {
            Some(s) => StdRng::seed_from_u64(s),
            None => StdRng::from_entropy(),
        };
        Self {
            rng: Arc::new(Mutex::new(rng)),
            generated_ids: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    pub fn generate_id(&self) -> u64 {
        let mut rng = self.rng.lock().unwrap();
        let mut generated_ids = self.generated_ids.lock().unwrap();
        loop {
            let id = rng.gen::<u64>();
            if !generated_ids.contains(&id) {
                generated_ids.insert(id);
                return id;
            }
        }
    }
}

/// 生成指定类型的模板代码（示例）
#[allow(dead_code)]
pub fn generate_wallet_template(name: &str) -> String {
    format!("// 钱包模板: {}\nstruct {}Wallet {{\n    // ...字段定义\n}}", name, name)
}

// 可扩展：生成密钥、配置、测试等脚手架代码

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_generate_id_unique() {
        let generator = IdGenerator::new(Some(42));
        let id1 = generator.generate_id();
        let id2 = generator.generate_id();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_generate_id_concurrent() {
        let generator = Arc::new(IdGenerator::new(None));
        let mut handles = vec![];
        for _ in 0..10 {
            let gen = Arc::clone(&generator);
            handles.push(thread::spawn(move || { gen.generate_id() }));
        }
        let mut ids = HashSet::new();
        for handle in handles {
            let id = handle.join().unwrap();
            assert!(ids.insert(id));
        }
    }

    #[test]
    fn test_generate_id_with_seed() {
        let generator1 = IdGenerator::new(Some(123));
        let generator2 = IdGenerator::new(Some(123));
        assert_eq!(generator1.generate_id(), generator2.generate_id());
    }
}
