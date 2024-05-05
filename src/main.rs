// src/main.rs

use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use no_comment::{IntoWithoutComments as _, languages};
mod config;

fn main() {
    // Get the current working directory
    let current_dir = env::current_dir().expect("Failed to get current directory");
    let current_dir_str = current_dir.to_str().expect("Failed to convert current directory to string");

    // Load or create the configuration
    let config = match config::load_config(current_dir_str) {
        Ok(config) => config,
        Err(_) => {
            println!("Configuration not found for the current directory.");
            println!("Let's create a new configuration.");

            let stdout = io::stdout();
            let stdin = io::stdin();

            let config = config::create_config(current_dir_str, stdin.lock(), stdout).expect("Failed to create configuration");
            config::save_config(&config, current_dir_str).expect("Failed to save configuration");
            config
        }
    };

    // Prompt the user for a specific goal or feature
    println!("Enter a specific goal or feature for the project:");
    let mut goal = String::new();
    io::stdin().read_line(&mut goal).expect("Failed to read goal");
    let goal = goal.trim();

    // Generate the prompt file
    let output_path = Path::new(&config.output_path);
    let project_name = &config.project_name;
    let current_date = chrono::Local::now().format("%Y%m%d").to_string();
    let prompt_filename = format!("{}_{}.txt", project_name, current_date);
    let prompt_path = output_path.join(prompt_filename);

    let mut prompt_file = fs::File::create(&prompt_path).expect("Failed to create prompt file");

    // Write the introductory prompt
    writeln!(prompt_file, "{}", config.intro_prompt).expect("Failed to write introductory prompt");

    // Write the tree representation of files matching allowed extensions
    let allowed_extensions: Vec<&str> = config.allowed_extensions.iter().map(|s| s.as_str()).collect();
    let tree_output = generate_tree_output(&current_dir, &allowed_extensions);
    writeln!(prompt_file, "{}", tree_output).expect("Failed to write tree output");

    // Write the contents of each file with comments removed
    for entry in fs::read_dir(&current_dir).expect("Failed to read directory") {
        let entry = entry.expect("Failed to get directory entry");
        let path = entry.path();
        if path.is_file() {
            let extension = path.extension().and_then(|ext| ext.to_str());
            if extension.map_or(false, |ext| allowed_extensions.contains(&ext)) {
                let file_content = fs::read_to_string(&path).expect("Failed to read file");
                let without_comments = remove_comments(&file_content, extension.unwrap());
                writeln!(prompt_file, "File: {}", path.display()).expect("Failed to write file path");
                writeln!(prompt_file, "{}", without_comments).expect("Failed to write file content");
            }
        }
    }

    // Write the specific goal
    writeln!(prompt_file, "Specific Goal: {}", goal).expect("Failed to write specific goal");

    // Update the configuration history
    let mut updated_config = config.clone();
    updated_config.history.push(goal.to_string());
    config::save_config(&updated_config, current_dir_str).expect("Failed to save updated configuration");

    println!("Prompt file generated: {}", prompt_path.display());
}

fn generate_tree_output(dir: &Path, allowed_extensions: &[&str]) -> String {
    let mut result = String::new();
    if dir.is_dir() {
        // Start the tree with the root directory
        result.push_str(&format!("{}\n", dir.display()));
        // Recursively build the tree
        if let Err(e) = visit_dirs(dir, "", allowed_extensions, &mut result) {
            eprintln!("Error: {}", e);
        }
    }
    result
}

fn visit_dirs(dir: &Path, prefix: &str, allowed_extensions: &[&str], result: &mut String) -> io::Result<()> {
    let mut entries = fs::read_dir(dir)?
        .map(|res| res.map(|e| e))
        .collect::<Result<Vec<_>, io::Error>>()?;

    // Sort entries by name to ensure consistent order
    entries.sort_by_key(|dir| dir.path());

    let count = entries.len();
    for (i, entry) in entries.iter().enumerate() {
        let path = entry.path();
        let file_name = path.file_name().unwrap().to_string_lossy().into_owned();
        let new_prefix = if i == count - 1 { "└── " } else { "├── " };

        if path.is_dir() {
            // Directory: recursively visit it
            result.push_str(&format!("{}{}{}", prefix, new_prefix, file_name));
            result.push('\n');
            visit_dirs(&path, &format!("{}    ", prefix), allowed_extensions, result)?;
        } else if let Some(ext) = path.extension() {
            // File: add it if it has an allowed extension
            println!("path extension: {}", ext.to_str().unwrap());
            if allowed_extensions.iter().any(|&e| ext.to_str() == Some(e)) {
                result.push_str(&format!("{}{}{}", prefix, new_prefix, file_name));
                result.push('\n');
            }
        }
    }
    Ok(())
}

/// Removes comments from the file content based on the file extension.
///
/// # Arguments
/// * `file_content` - The content of the file as a string.
/// * `extension` - The file extension indicating the programming language (e.g., "rs", "c", "py").
///
/// # Returns
/// A new string with comments removed, according to the syntax of the specified programming language.
fn remove_comments(file_content: &str, extension: &str) -> String {
    match extension {
        "rs" => file_content
            .chars()
            .without_comments(languages::rust())
            .collect::<String>(),
        "c" => file_content
            .chars()
            .without_comments(languages::c())
            .collect::<String>(),
        "py" => file_content
            .chars()
            .without_comments(languages::python())
            .collect::<String>(),
        _ => file_content.to_string(), // If the extension is not recognized, return the original content.
    }
}