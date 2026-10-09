// Sample source code referenced by documentation
pub struct BotoxConfig {
    pub theme: String,
    pub columns: usize,
    pub bibliography: bool,
}

impl BotoxConfig {
    pub fn default_document() -> Self {
        Self {
            theme: "academic".to_string(),
            columns: 1,
            bibliography: true,
        }
    }
}

fn main() {
    let _cfg = BotoxConfig::default_document();
}
