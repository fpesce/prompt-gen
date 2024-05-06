# Prompt Generator

`prompt-gen` is a command-line tool written in Rust that automates the process of generating prompts for large language models (LLMs) on extensive coding projects. It is designed to improve productivity by creating structured prompts that include relevant project information, file structure, and code snippets.

## Features

- Loads project-specific configurations from a TOML file located in the user's home directory
- Supports multiple project configurations based on the current working directory
- Generates a new configuration for the current project if one doesn't exist
- Includes an introductory prompt explaining the project and codebase
- Allows specifying allowed file extensions and directories to ignore
- Generates a prompt file with the following structure:
  - Introductory prompt
  - Tree representation of project files matching allowed extensions
  - Content of each file with comments removed
  - Specific goal or feature requested by the user
- Removes comments from code files using the `no-comment` crate
- Updates the configuration file with the history of previous prompts

## Installation

To install `prompt-gen`, ensure you have Rust installed on your system. Then, clone the repository and build the project:

```bash
git clone https://github.com/yourusername/prompt-gen.git
cd prompt-gen
cargo build --release
```

The compiled binary will be located at `target/release/prompt-gen`.

## Usage

To generate a prompt for your project, navigate to the project directory and run the `prompt-gen` command:

```bash
path/to/prompt-gen
```

If a configuration file doesn't exist for the current project, `prompt-gen` will guide you through creating one. You'll be asked to provide the following information:

- Project name
- Output path for the generated prompt file
- Introductory prompt explaining the project and codebase
- Allowed file extensions (comma-separated)
- Directories to ignore (comma-separated)

Once the configuration is loaded or created, you'll be prompted to enter a specific goal or feature for the project. `prompt-gen` will then generate a prompt file in the specified output directory with the following format: `project_name_YYMMDD.txt`.

The generated prompt file will include:

- The introductory prompt
- A tree representation of project files matching the allowed extensions
- The content of each file with comments removed
- The specific goal or feature you entered

## Configuration

`prompt-gen` uses a TOML file for configuration, located in the user's home directory with the name `.prompt-gen.toml`. The configuration file stores project-specific settings, with each project identified by its directory path.

Example configuration:

```toml
["/path/to/project1"]
project_name = "Project 1"
output_path = "/path/to/output1"
intro_prompt = "Intro prompt for Project 1"
allowed_extensions = ["rs", "toml"]
deny_dirs = ["target"]
history = ["Goal 1", "Goal 2"]

["/path/to/project2"]
project_name = "Project 2"
output_path = "/path/to/output2"
intro_prompt = "Intro prompt for Project 2"
allowed_extensions = ["rs", "md"]
deny_dirs = ["dist", "build"]
history = ["Goal 3"]
```

## Dependencies

- `dirs`: For accessing the user's home directory path
- `serde`: For serializing and deserializing configuration data
- `toml`: For parsing and generating TOML files
- `chrono`: For formatting dates in the generated prompt filename
- `no-comment`: For removing comments from code files

## Contributing

Contributions are welcome! If you find a bug or have a feature request, please open an issue on the GitHub repository. If you'd like to contribute code, please fork the repository and submit a pull request.

## License

This project is licensed under the [MIT License](LICENSE).