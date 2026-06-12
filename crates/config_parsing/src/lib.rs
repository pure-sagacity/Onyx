use serde::{Deserialize, Serialize};
use std::fs;
use std::path;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    pub version: String,
    pub project_id: String,
    pub environment: String,

    pub shell: Option<ShellConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ShellConfig {
    pub default: String,
}

impl Config {
    /// This function loads the configuration from a TOML file from the hidden folder inside the project dir.
    pub fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let path = path::Path::new(path);
        let path = path.join(".onyx").join("config.toml");

        let config_content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&config_content)?;

        Ok(config)
    }

    /// This function checks if the configuration file exists in the hidden folder inside the project dir.
    pub fn config_exists(path: &str) -> bool {
        let path = path::Path::new(path);
        let config_path = path.join(".onyx");

        for file in ["config.toml"] {
            let file_path = config_path.join(file);
            if !file_path.exists() {
                return false;
            }
        }

        config_path.exists()
    }

    /// This function creates the configuration file based on user given input
    pub fn create_config_file(
        project_id: &str,
        environment: &str,
        shell: Option<ShellConfig>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let config = Config {
            version: "1.0".to_string(),
            project_id: project_id.to_string(),
            environment: environment.to_string(),
            shell,
        };

        let config_content = toml::to_string_pretty(&config)?;
        let config_path = path::Path::new(".onyx").join("config.toml");

        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(config_path, config_content)?;

        Ok(())
    }
}
