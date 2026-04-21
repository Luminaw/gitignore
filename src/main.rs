use clap::Parser;
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
struct Args {
    action: String,
    name: Option<String>,
}

fn get_config_dir() -> PathBuf {
    let home = dirs::home_dir().expect("Could not find home directory");
    home.join(".config").join("gitignore")
}

fn main() {
    let args = Args::parse();
    match args.action.as_str() {
        "add" => {
            let name = match args.name {
                Some(n) => n,
                None => {
                    eprintln!("Error: 'add' command requires a template name.");
                    return;
                }
            };

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

        "list" => {
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
        "create" => {
            let source_path_str = match args.name {
                Some(n) => n,
                None => {
                    eprintln!("Error: 'create' command requires a source file path.");
                    return;
                }
            };

            let source_path = Path::new(&source_path_str);
            if !source_path.exists() {
                eprintln!("Error: Source file '{}' not found.", source_path_str);
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
        _ => println!("Unknown action"),
    }
}

fn append_to_gitignore(path: &Path, content: &str) -> Result<(), std::io::Error> {
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    file.write_all(format!("\n\n{}", content).as_bytes())?;
    Ok(())
}
