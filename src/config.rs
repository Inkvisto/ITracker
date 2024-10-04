use config::{Config, File, FileFormat};
use serde::{Deserialize, Serialize};
use std::fs;

/// Structure representing the configuration data.
#[derive(Debug, Deserialize, Serialize)]
pub struct ConfigData {
    /// Optional output file path.
    pub output_file: Option<String>,
}

/// Loads the configuration from the specified config file.
///
/// This function attempts to read a TOML configuration file named `config`
/// (or `config.toml` if the specified file is required) and deserialize it
/// into a `ConfigData` struct. If the file does not exist, it will return
/// an error unless the file is marked as optional.
///
/// # Returns
/// - `Ok(ConfigData)`: The loaded configuration data if successful.
/// - `Err(Box<dyn std::error::Error>)`: An error if the loading or deserialization fails.
pub fn load_config() -> Result<ConfigData, Box<dyn std::error::Error>> {
    let config = Config::builder()
        .add_source(File::new("config", FileFormat::Toml).required(false))
        .build()?
        .try_deserialize::<ConfigData>()?;

    Ok(config)
}

/// Saves the updated configuration to the config file.
///
/// This function serializes the given `ConfigData` struct into TOML format
/// and writes it to a file named `config.toml`. If the file already exists,
/// it will be overwritten.
///
/// # Arguments
/// - `config`: A reference to the `ConfigData` struct that needs to be saved.
///
/// # Returns
/// - `Ok(())`: If the saving process is successful.
/// - `Err(Box<dyn std::error::Error>)`: An error if the serialization or writing fails.
pub fn save_config(config: &ConfigData) -> Result<(), Box<dyn std::error::Error>> {
    // Serialize config into TOML format
    let toml_str = toml::to_string(&config)?;

    // Write the serialized config back to the config file
    fs::write("config.toml", toml_str)?;

    Ok(())
}
