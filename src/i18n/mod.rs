use anyhow::Result;
use fluent::{FluentBundle, FluentResource};
use fluent_bundle::FluentArgs;
use std::collections::HashMap;
use tracing::{debug, info, warn};

pub mod localization;

pub struct I18nManager {
    bundles: HashMap<String, FluentBundle<FluentResource>>,
    default_language: String,
}

impl I18nManager {
    pub fn new(default_language: String) -> Self {
        info!("馃實 Initializing internationalization manager (default: {})", default_language);

        Self { bundles: HashMap::new(), default_language }
    }

    pub fn load_language(&mut self, language: &str, content: &str) -> Result<()> {
        debug!("Loading language: {}", language);

        let resource = FluentResource::try_new(content.to_string())
            .map_err(|e| anyhow::anyhow!("Failed to parse Fluent resource: {:?}", e))?;

        let mut bundle = FluentBundle::new(vec![language
            .parse()
            .map_err(|e| anyhow::anyhow!("Invalid language code: {}", e))?]);
        bundle.set_use_isolating(false); // 淇锛氬叧闂?Unicode 闅旂鍖呰

        bundle
            .add_resource(resource)
            .map_err(|e| anyhow::anyhow!("Failed to add resource to bundle: {:?}", e))?;

        self.bundles.insert(language.to_string(), bundle);

        info!("鉁?Loaded language: {}", language);
        Ok(())
    }

    pub fn get_text(&self, language: &str, key: &str, args: Option<&FluentArgs>) -> String {
        let bundle =
            self.bundles.get(language).or_else(|| self.bundles.get(&self.default_language));

        match bundle {
            Some(bundle) => {
                let message = bundle.get_message(key);
                match message {
                    Some(message) => {
                        let pattern = message.value().unwrap_or_else(|| {
                            warn!("Message '{}' has no value", key);
                            message.attributes().next().map(|attr| attr.value()).unwrap_or_else(
                                || message.value().expect("Message has no value or attributes"),
                            )
                        });

                        let mut errors = vec![];
                        let result = bundle.format_pattern(pattern, args, &mut errors);

                        if !errors.is_empty() {
                            warn!("Errors formatting message '{}': {:?}", key, errors);
                        }

                        result.to_string()
                    }
                    None => {
                        warn!("Message '{}' not found in language '{}'", key, language);
                        key.to_string()
                    }
                }
            }
            None => {
                warn!("Language '{}' not loaded, fallback to key", language);
                key.to_string()
            }
        }
    }

    pub fn get_supported_languages(&self) -> Vec<String> {
        self.bundles.keys().cloned().collect()
    }

    pub fn is_language_supported(&self, language: &str) -> bool {
        self.bundles.contains_key(language)
    }
}

pub fn init_default_languages() -> Result<I18nManager> {
    let mut manager = I18nManager::new("en".to_string());

    // Load English
    let en_content = include_str!("../../resources/i18n/en.ftl");
    manager.load_language("en", en_content)?;

    // Load Chinese
    let zh_content = include_str!("../../resources/i18n/zh.ftl");
    manager.load_language("zh", zh_content)?;

    Ok(manager)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fluent_bundle::FluentArgs;

    #[test]
    fn test_i18n_manager() {
        let mut manager = I18nManager::new("en".to_string());

        let en_content = r#"
hello = Hello, World!
greeting = Hello, { $name }!
"#;

        let zh_content = r#"
hello = 浣犲ソ锛屼笘鐣岋紒
greeting = 浣犲ソ锛寋 $name }锛?"#;

        manager.load_language("en", en_content).unwrap();
        manager.load_language("zh", zh_content).unwrap();

        // Test simple message
        assert_eq!(manager.get_text("en", "hello", None), "Hello, World!");
        assert_eq!(manager.get_text("zh", "hello", None), "浣犲ソ锛屼笘鐣岋紒");

        // Test message with arguments
        let mut args = FluentArgs::new();
        args.set("name", "Alice");

        assert_eq!(manager.get_text("en", "greeting", Some(&args)), "Hello, Alice!");
        assert_eq!(manager.get_text("zh", "greeting", Some(&args)), "浣犲ソ锛孉lice锛?);

        // Test fallback to default language
        assert_eq!(manager.get_text("fr", "hello", None), "Hello, World!");

        // Test missing key
        assert_eq!(manager.get_text("en", "missing", None), "missing");
    }
}
