use serde::Deserialize;
use serde::Deserializer;
use std::env::current_dir;
use std::path::PathBuf;

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
    #[serde(
        rename(deserialize = "template_directory"),
        deserialize_with = "deserialize_template_dir",
        default = "default_template_dir"
    )]
    pub template_dir: PathBuf,
}

fn default_template_dir() -> PathBuf {
    current_dir()
        .unwrap()
        .join("templates")
        .canonicalize()
        .unwrap()
        .join("**")
}

fn deserialize_template_dir<'de, D>(deserializer: D) -> Result<PathBuf, D::Error>
where
    D: Deserializer<'de>,
{
    let mut buf = PathBuf::deserialize(deserializer)?;
    if buf.is_relative() {
        buf = current_dir().unwrap().join(buf);
    }
    Ok(buf.canonicalize().unwrap().join("**"))
}
