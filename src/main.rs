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
    let deny_directories: Vec<&str> = config.deny_dirs.iter().map(|s| s.as_str()).collect();
    let tree_output = generate_tree_output(&current_dir, &allowed_extensions, &deny_directories, &mut prompt_file);
    writeln!(prompt_file, "{}", tree_output).expect("Failed to write tree output");

    // Write the specific goal
    writeln!(prompt_file, "Specific Goal: {}", goal).expect("Failed to write specific goal");

    // Update the configuration history
    let mut updated_config = config.clone();
    updated_config.history.push(goal.to_string());
    config::save_config(&updated_config, current_dir_str).expect("Failed to save updated configuration");

    println!("Prompt file generated: {}", prompt_path.display());
}

fn generate_tree_output(dir: &Path, allowed_extensions: &[&str], deny_dirs: &[&str], prompt_file: &mut fs::File) -> String {
    let mut result = String::new();
    if dir.is_dir() {
        // Start the tree with the root directory
        result.push_str(&format!("{}\n", dir.display()));
        // Recursively build the tree
        if let Err(e) = visit_dirs(dir, "", allowed_extensions, deny_dirs, prompt_file, &mut result) {
            eprintln!("Error: {}", e);
        }
    }
    result
}

fn visit_dirs(dir: &Path, prefix: &str, allowed_extensions: &[&str], deny_dirs: &[&str], prompt_file: &mut fs::File, result: &mut String) -> io::Result<()> {
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
            if deny_dirs.iter().any(|&e| file_name == e) {
                continue;
            }
            // Directory: recursively visit it
            result.push_str(&format!("{}{}{}", prefix, new_prefix, file_name));
            result.push('\n');
            visit_dirs(&path, &format!("{}    ", prefix), allowed_extensions, deny_dirs, prompt_file, result)?;
        } else if let Some(ext) = path.extension() {
            // File: add it if it has an allowed extension
            if allowed_extensions.iter().any(|&e| ext.to_str() == Some(e)) {
                result.push_str(&format!("{}{}{}", prefix, new_prefix, file_name));
                result.push('\n');

// Read the file content and remove comments
                let file_content = fs::read_to_string(&path)?;
                let without_comments = remove_comments(&file_content, ext.to_str().unwrap());
                let cleaned_content = remove_empty_lines(&without_comments);

                let relative_path = path.strip_prefix(&env::current_dir().unwrap()).unwrap();
                writeln!(prompt_file, "File: {}", relative_path.display())?;
                writeln!(prompt_file, "```")?;
                writeln!(prompt_file, "{}", cleaned_content)?;
                writeln!(prompt_file, "```")?;
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

fn remove_empty_lines(input: &str) -> String {
    input
        .lines()
        .filter(|line| !line.trim().is_empty())  // Filter out empty or whitespace-only lines
        .collect::<Vec<&str>>()  // Collect lines back into a Vec
        .join("\n")  // Join them into a single string with newline characters
}
