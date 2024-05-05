// src/config.rs

use std::env;
use std::fs;
use std::io::{self};
use std::path::PathBuf;

use dirs::home_dir;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub project_name: String,
    pub output_path: String,
    pub intro_prompt: String,
    pub allowed_extensions: Vec<String>,
    pub deny_dirs: Vec<String>,
    pub history: Vec<String>,
}

pub fn get_config_path() -> Option<PathBuf> {
    if let Some(home_dir) = home_dir() {
        let suffix = env::var("CONFIG_TEST_SUFFIX").unwrap_or_default();
        let config_filename = format!(".prompt-gen{}.toml", suffix);
        let config_path = home_dir.join(config_filename);
        println!("Config Path: {}", config_path.display());
        Some(config_path)
    } else {
        None
    }
}

pub fn load_config(current_dir: &str) -> Result<Config, Box<dyn std::error::Error>> {
    if let Some(config_path) = get_config_path() {
        if config_path.exists() {
            let config_content = fs::read_to_string(config_path)?;
            println!("Read TOML from file: {}", config_content);  // Print the content read from file
            let config_table: toml::Table = toml::from_str(&config_content)?;

            if let Some(project_config) = config_table.get(current_dir) {
                let project_config: Config = project_config.clone().try_into()?;
                Ok(project_config)
            } else {
                Err(format!("Configuration not found for directory: {}", current_dir).into())
            }
        } else {
            Err("Configuration file not found.".into())
        }
    } else {
        Err("Home directory not found.".into())
    }
}

pub fn create_config<R, W>(current_dir: &str, mut reader: R, mut writer: W) -> Result<Config, Box<dyn std::error::Error>>
    where
        R: io::BufRead,
        W: io::Write,
{
    writeln!(writer, "Configuration not found for the current directory.")?;
    writeln!(writer, "Let's create a new configuration.")?;

    write!(writer, "Enter the project name (default: {}): ", current_dir)?;
    writer.flush()?;
    let mut project_name = String::new();
    reader.read_line(&mut project_name)?;
    let project_name = project_name.trim();
    let project_name = if project_name.is_empty() {
        current_dir.to_string()
    } else {
        project_name.to_string()
    };

    write!(writer, "Enter the output path: ")?;
    writer.flush()?;
    let mut output_path = String::new();
    reader.read_line(&mut output_path)?;
    let output_path = output_path.trim().to_string();

    write!(writer, "Enter the introductory prompt: ")?;
    writer.flush()?;
    let mut intro_prompt = String::new();
    reader.read_line(&mut intro_prompt)?;
    let intro_prompt = intro_prompt.trim().to_string();

    write!(writer, "Enter the allowed file extensions (comma-separated): ")?;
    writer.flush()?;
    let mut allowed_extensions = String::new();
    reader.read_line(&mut allowed_extensions)?;
    let allowed_extensions: Vec<String> = allowed_extensions
        .trim()
        .split(',')
        .map(|ext| ext.trim().to_string())
        .collect();

    write!(writer, "Enter the directories to ignore (comma-separated): ")?;
    writer.flush()?;
    let mut deny_dirs = String::new();
    reader.read_line(&mut deny_dirs)?;
    let deny_dirs: Vec<String> = deny_dirs
        .trim()
        .split(',')
        .map(|dir| dir.trim().to_string())
        .collect();

    let config = Config {
        project_name,
        output_path,
        intro_prompt,
        allowed_extensions,
        deny_dirs,
        history: Vec::new(),
    };

    Ok(config)
}

pub fn save_config(config: &Config, current_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(config_path) = get_config_path() {
        let mut config_content = if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            toml::from_str(&content)?
        } else {
            toml::Table::new()
        };

        let config_value = toml::Value::try_from(config)?;
        config_content.insert(current_dir.to_string(), config_value);

        let config_str = toml::to_string(&config_content)?;
        println!("Writing TOML to file: {}", config_str);  // Print the TOML string being written
        fs::write(config_path, config_str)?;
        Ok(())
    } else {
        Err("Home directory not found.".into())
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    /* This closure allows to have test using a different toml file, this is important because cargo test default to being multithreaded and concurrent access to a file would fail. */
    fn with_test_env<F: FnOnce() -> ()>(test_name: &str, test: F) {
        // Setup: Set the environment variable
        std::env::remove_var("CONFIG_TEST_SUFFIX");
        std::env::set_var("CONFIG_TEST_SUFFIX", format!("-{}", test_name));

        // Run the test
        test();

        // Teardown: Remove the environment variable
        std::env::remove_var("CONFIG_TEST_SUFFIX");
    }

    #[test]
    fn test_get_config_path() {
        let config_path = get_config_path();
        assert!(config_path.is_some());
        let config_path = config_path.unwrap();
        println!("Config path: {}", config_path.display());
        assert!(config_path.ends_with(".prompt-gen.toml"));
    }

    #[test]
    fn test_load_existing_config() {
        with_test_env("test_load_existing_config", || {
            let test_dir = "/path/to/test/dir";
            let project_name = "Test Project";
            let output_path = "/path/to/output";
            let intro_prompt = "Test intro prompt";
            let allowed_extensions = "rs,toml";
            let deny_dirs = "target,node_modules";

            let input = format!(
                "{}\n{}\n{}\n{}\n{}\n",
                project_name, output_path, intro_prompt, allowed_extensions, deny_dirs
            );
            let mut reader = io::BufReader::new(input.as_bytes());
            let mut writer = Vec::new();

            let created_config = create_config(test_dir, &mut reader, &mut writer).unwrap();

            println!("test_load_existing_config: {:?}", created_config);

            save_config(&created_config, test_dir).unwrap();

            // Load the config and verify its contents
            let loaded_config = load_config(test_dir).unwrap();
            assert_eq!(loaded_config.project_name, project_name);
            assert_eq!(loaded_config.output_path, output_path);
            assert_eq!(loaded_config.intro_prompt, intro_prompt);
            assert_eq!(loaded_config.allowed_extensions, vec!["rs", "toml"]);
            assert_eq!(loaded_config.deny_dirs, vec!["target", "node_modules"]);
            assert!(loaded_config.history.is_empty());

            // Clean up the temporary test config file
            let config_path = get_config_path().unwrap();
            fs::remove_file(config_path).unwrap();
        });
    }

    #[test]
    fn test_create_new_config() {
        with_test_env("test_create_new_config", || {
            let current_dir = "/path/to/current/dir";
            let project_name = "New Project";
            let output_path = "/path/to/new/output";
            let intro_prompt = "New intro prompt";
            let allowed_extensions = "rs,toml,md";
            let deny_dirs = "target,node_modules";

            let input = format!(
                "{}\n{}\n{}\n{}\n{}\n",
                project_name, output_path, intro_prompt, allowed_extensions, deny_dirs
            );
            let mut reader = io::BufReader::new(input.as_bytes());
            let mut writer = Vec::new();

            let created_config = create_config(current_dir, &mut reader, &mut writer).unwrap();
            save_config(&created_config, current_dir).unwrap();

            // Load the config and verify its contents
            let loaded_config = load_config(current_dir).unwrap();
            assert_eq!(loaded_config.project_name, project_name);
            assert_eq!(loaded_config.output_path, output_path);
            assert_eq!(loaded_config.intro_prompt, intro_prompt);
            assert_eq!(
                loaded_config.allowed_extensions,
                vec!["rs", "toml", "md"]
            );
            assert_eq!(loaded_config.deny_dirs, vec!["target", "node_modules"]);
            assert!(loaded_config.history.is_empty());

            let output = String::from_utf8(writer).unwrap();
            assert!(output.contains("Configuration not found for the current directory."));
            assert!(output.contains("Let's create a new configuration."));
            assert!(output.contains(&format!("Enter the project name (default: {}): ", current_dir)));
            assert!(output.contains("Enter the output path: "));
            assert!(output.contains("Enter the introductory prompt: "));
            assert!(output.contains("Enter the allowed file extensions (comma-separated): "));
            assert!(output.contains("Enter the directories to ignore (comma-separated): "));

            // Clean up the temporary test config file
            let config_path = get_config_path().unwrap();
            fs::remove_file(config_path).unwrap();
        });
    }

    #[test]
    fn test_load_multiple_configs() {
        with_test_env("test_load_multiple_configs", || {
            let current_dir1 = "/path/to/project1";
            let current_dir2 = "/path/to/project2";

            let input1 = "Project 1\n/path/to/output1\nIntro prompt for Project 1\nrs,toml\ntarget\n".to_string();
            let mut reader1 = io::BufReader::new(input1.as_bytes());
            let mut writer1 = Vec::new();

            let input2 = "Project 2\n/path/to/output2\nIntro prompt for Project 2\nrs,md\ndist,build\n".to_string();
            let mut reader2 = io::BufReader::new(input2.as_bytes());
            let mut writer2 = Vec::new();

            let created_config1 = create_config(current_dir1, &mut reader1, &mut writer1).unwrap();
            save_config(&created_config1, current_dir1).unwrap();

            let created_config2 = create_config(current_dir2, &mut reader2, &mut writer2).unwrap();
            save_config(&created_config2, current_dir2).unwrap();

            // Load the configs and verify their contents
            let loaded_config1 = load_config(current_dir1).unwrap();
            assert_eq!(loaded_config1.project_name, "Project 1");
            assert_eq!(loaded_config1.output_path, "/path/to/output1");
            assert_eq!(loaded_config1.intro_prompt, "Intro prompt for Project 1");
            assert_eq!(loaded_config1.allowed_extensions, vec!["rs", "toml"]);
            assert_eq!(loaded_config1.deny_dirs, vec!["target"]);
            assert!(loaded_config1.history.is_empty());

            let loaded_config2 = load_config(current_dir2).unwrap();
            assert_eq!(loaded_config2.project_name, "Project 2");
            assert_eq!(loaded_config2.output_path, "/path/to/output2");
            assert_eq!(loaded_config2.intro_prompt, "Intro prompt for Project 2");
            assert_eq!(loaded_config2.allowed_extensions, vec!["rs", "md"]);
            assert_eq!(loaded_config2.deny_dirs, vec!["dist", "build"]);
            assert!(loaded_config2.history.is_empty());

            // Test loading a non-existent configuration
            let non_existent_dir = "/path/to/non-existent-dir";
            match load_config(non_existent_dir) {
                Ok(_) => panic!("Expected an error, but got an Ok result"),
                Err(e) => assert_eq!(
                    format!("Configuration not found for directory: {}", non_existent_dir),
                    e.to_string()
                ),
            }

            // Clean up the temporary test config file
            let config_path = get_config_path().unwrap();
            fs::remove_file(config_path).unwrap();
        });
    }
}