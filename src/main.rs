use clap::{Parser, Subcommand};
use rust_embed::RustEmbed;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(RustEmbed)]
#[folder = "ignores/"] // A folder in your project root containing the .gitignore files
#[include = "*.gitignore"]
#[include = "**/*.gitignore"]
struct GitignoreAssets;

#[derive(Parser)]
#[command(name = "gitignore")]
#[command(author = "Luminaw")]
#[command(version = "0.1.0")]
#[command(about = "A simple gitignore generator CLI", long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a gitignore template to the current directory
    Add {
        /// The name of the template to add (e.g., Rust, Python)
        name: String,
    },
    /// List all available templates
    List,
    /// Create a custom template from a local file
    Create {
        /// The path to the local file to use as a template
        path: String,
    },
}

fn get_config_dir() -> PathBuf {
    let home = dirs::home_dir().expect("Could not find home directory");
    home.join(".config").join("gitignore")
}

fn main() {
    let args = Args::parse();
    match args.command {
        Commands::Add { name } => {
            let template_name = if name.ends_with(".gitignore") {
                name.clone()
            } else {
                format!("{}.gitignore", name)
            };

            // 1. Try embedded assets
            let mut content = GitignoreAssets::get(&template_name)
                .map(|a| std::str::from_utf8(a.data.as_ref()).expect("Invalid UTF-8").to_string());

            // 2. Try custom config dir
            if content.is_none() {
                let custom_path = get_config_dir().join(&template_name);
                if custom_path.exists() {
                    content = Some(fs::read_to_string(custom_path).expect("Failed to read custom template"));
                }
            }

            // 3. Fallback: try without extension if it wasn't provided (for embedded)
            if content.is_none() && !name.ends_with(".gitignore") {
                content = GitignoreAssets::get(&name)
                    .map(|a| std::str::from_utf8(a.data.as_ref()).expect("Invalid UTF-8").to_string());
                
                if content.is_none() {
                    let custom_path = get_config_dir().join(&name);
                    if custom_path.exists() {
                        content = Some(fs::read_to_string(custom_path).expect("Failed to read custom template"));
                    }
                }
            }

            let template_content = match content {
                Some(c) => c,
                None => {
                    eprintln!("Error: Template '{}' not found in embedded or custom templates.", name);
                    return;
                }
            };

            let target_path = Path::new(".gitignore");
            if target_path.exists() {
                let local_content =
                    fs::read_to_string(target_path).expect("Failed to read existing .gitignore");
                if local_content.trim() == template_content.trim() {
                    eprintln!("Error: The .gitignore you are trying to add already exists and is identical.");
                    return;
                }
                append_to_gitignore(target_path, &template_content).expect("Failed to append to .gitignore");
                println!("Appended {} to .gitignore", template_name);
            } else {
                fs::write(target_path, template_content).expect("Failed to create .gitignore");
                println!("Created .gitignore from {} template", template_name);
            }
        }

        Commands::List => {
            println!("--- Embedded Templates ---");
            for file in GitignoreAssets::iter() {
                println!("{}", file);
            }

            let config_dir = get_config_dir();
            if config_dir.exists() {
                println!("\n--- Custom Templates (~/.config/gitignore/) ---");
                if let Ok(entries) = fs::read_dir(config_dir) {
                    for entry in entries.flatten() {
                        if let Ok(file_type) = entry.file_type() {
                            if file_type.is_file() {
                                println!("{}", entry.file_name().to_string_lossy());
                            }
                        }
                    }
                }
            }
        }
        Commands::Create { path } => {
            let source_path = Path::new(&path);
            if !source_path.exists() {
                eprintln!("Error: Source file '{}' not found.", path);
                return;
            }

            let config_dir = get_config_dir();
            if !config_dir.exists() {
                fs::create_dir_all(&config_dir).expect("Failed to create config directory");
            }

            let filename = source_path.file_name().expect("Invalid filename");
            let dest_path = config_dir.join(filename);

            fs::copy(source_path, &dest_path).expect("Failed to copy template to config directory");
            println!("Successfully created custom template: {}", filename.to_string_lossy());
        }
    }
}

fn append_to_gitignore(path: &Path, content: &str) -> Result<(), std::io::Error> {
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    file.write_all(format!("\n\n{}", content).as_bytes())?;
    Ok(())
}
