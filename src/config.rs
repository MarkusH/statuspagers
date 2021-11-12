use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Backend {
    GITHUB,
    // GITLAB,
}

#[derive(Debug, Deserialize)]
pub struct GitHub {
    pub owner: String,
    pub repository: String,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub backend: Backend,
    pub components: Vec<String>,
    pub github: Option<GitHub>,
}
