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
        rename(deserialize = "output_directory"),
        deserialize_with = "deserialize_output_dir",
        default = "default_output_dir"
    )]
    pub output_dir: PathBuf,
    #[serde(
        rename(deserialize = "template_directory"),
        deserialize_with = "deserialize_template_dir",
        default = "default_template_dir"
    )]
    pub template_dir: PathBuf,
}

fn default_output_dir() -> PathBuf {
    let buf = current_dir().unwrap().join("html");
    if !buf.exists() {
        eprintln!("Path {:?} does not exist.", buf);
    }
    if !buf.is_dir() {
        eprintln!("Path {:?} is not a directory.", buf);
    }
    buf.canonicalize().unwrap()
}

fn deserialize_output_dir<'de, D>(deserializer: D) -> Result<PathBuf, D::Error>
where
    D: Deserializer<'de>,
{
    let mut buf = PathBuf::deserialize(deserializer)?;
    if buf.is_relative() {
        buf = current_dir().unwrap().join(buf);
    }
    if !buf.exists() {
        eprintln!("Path {:?} does not exist.", buf);
    }
    if !buf.is_dir() {
        eprintln!("Path {:?} is not a directory.", buf);
    }
    Ok(buf.canonicalize().unwrap())
}

fn default_template_dir() -> PathBuf {
    let buf = current_dir().unwrap().join("templates");
    if !buf.exists() {
        eprintln!("Path {:?} does not exist.", buf);
    }
    if !buf.is_dir() {
        eprintln!("Path {:?} is not a directory.", buf);
    }

    buf.canonicalize().unwrap().join("**")
}

fn deserialize_template_dir<'de, D>(deserializer: D) -> Result<PathBuf, D::Error>
where
    D: Deserializer<'de>,
{
    let mut buf = PathBuf::deserialize(deserializer)?;
    if buf.is_relative() {
        buf = current_dir().unwrap().join(buf);
    }
    if !buf.exists() {
        eprintln!("Path {:?} does not exist.", buf);
    }
    if !buf.is_dir() {
        eprintln!("Path {:?} is not a directory.", buf);
    }
    Ok(buf.canonicalize().unwrap().join("**"))
}
