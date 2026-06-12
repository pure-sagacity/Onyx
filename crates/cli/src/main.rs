use base64::{Engine, engine::general_purpose::STANDARD};
use clap::Parser;
use config_parsing::Config;
use crypto::gen_or_retrieve_key;
use dialoguer::{Select, theme::ColorfulTheme};
use rpassword::{ConfigBuilder, read_password_with_config};
use std::io::{self, Write};

#[derive(Parser, Debug)]
#[clap(name = "Onyx", version, about, long_about = None)]
struct Cli {
    /// The path to project folder
    #[clap(short, long, default_value = "./.onyx/")]
    project_path: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand, Debug, Clone)]
enum Commands {
    Init {},
    Set {
        name: String,
    },
    List {},
    Get {
        name: String,
    },
    Delete {
        name: String,
    },
    Inject {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        commands: Vec<String>,
    },
    Shell {
        #[arg(long)]
        shell: Option<String>,
    },
}

fn main() {
    let password_entry_config = ConfigBuilder::new().password_feedback_mask('*').build();

    let home_dir = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());

    // Prod
    // let DB_URL = format!("{}/.config/onyx/secrets.db", home_dir);
    // Dev
    let db_url = "./secrets.db".to_string();

    let cli = Cli::parse();
    let key = gen_or_retrieve_key().expect("Failed to generate or retrieve encryption key.");

    let db: database::Database = database::Database { url: db_url };

    match cli.command {
        Commands::Init {} => {
            print!("Enter project name: ");
            io::stdout().flush().unwrap();
            let mut project_name = String::new();
            io::stdin().read_line(&mut project_name).unwrap();

            print!("Enter project description:\n> ");
            io::stdout().flush().unwrap();
            let mut project_description = String::new();
            io::stdin().read_line(&mut project_description).unwrap();

            if (project_name.trim().is_empty()) {
                println!("Project name cannot be empty. Aborting.");
                return;
            }

            let project_description = if project_description.trim().is_empty() {
                None
            } else {
                Some(project_description.trim().to_string())
            };

            let selections = ["Dev", "Staging", "Production", "Other"];

            let selection = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Which environment is your project in?")
                .default(0)
                .items(&selections[..])
                .interact_opt()
                .expect("Failed to select environment.");

            let environment = match selection {
                Some(index) => {
                    let environment = selections[index];
                    println!("Selected environment: {}", environment);

                    match environment {
                        "Dev" => "Dev".to_string(),
                        "Staging" => "Staging".to_string(),
                        "Production" => "Production".to_string(),
                        "Other" => {
                            print!("Enter custom environment name: ");
                            io::stdout().flush().unwrap();
                            let mut custom_env = String::new();
                            io::stdin().read_line(&mut custom_env).unwrap();
                            // Trim and convert the resulting &str into an owned String
                            custom_env.trim().to_string()
                        }
                        _ => "Dev".to_string(),
                    }
                }
                None => "Dev".to_string(),
            };

            let project = db
                .create_project(project_name, environment.clone(), project_description)
                .expect("Failed to create the project, please try again.");

            Config::create_config_file(&project.id, &environment.to_lowercase(), None)
                .expect("Failed to create configuration files.");

            println!("Project created successfully.");
        }
        Commands::Set { name } => {
            // 1. Attempt to fetch the secret immediately
            let existing_secret = db
                .get_secret_by(database::SecretField::Name, &name)
                .expect("Database error while retrieving secret.");

            match existing_secret {
                Some(secret) => {
                    // --- OVERWRITE LOGIC ---
                    print!("Secret already exists. Do you want to overwrite it? (y/N): ");
                    io::stdout().flush().unwrap();

                    let mut input = String::new();
                    io::stdin().read_line(&mut input).unwrap();
                    if input.trim().to_lowercase() != "y" {
                        println!("Aborting.");
                        return;
                    }

                    print!("Enter secret value: ");
                    io::stdout().flush().unwrap();

                    let value = read_password_with_config(password_entry_config)
                        .expect("Failed to read secret value.");

                    let (nonce, ciphertext) = crypto::encrypt(value, &key);
                    let encrypted_value = STANDARD.encode(ciphertext);
                    let new_nonce = STANDARD.encode(nonce);

                    db.set_secret(
                        secret.id.expect("Failed to retrieve secret ID"),
                        encrypted_value,
                        new_nonce,
                    )
                    .expect("Failed to update secret.");
                }
                None => {
                    // --- NEW SECRET LOGIC ---
                    print!("Enter secret value: ");
                    io::stdout().flush().unwrap();

                    let value = read_password_with_config(password_entry_config)
                        .expect("Failed to read secret value.");

                    let (nonce, ciphertext) = crypto::encrypt(value, &key);
                    let encrypted_value = STANDARD.encode(ciphertext);
                    let new_nonce = STANDARD.encode(nonce);

                    let fake_uuid = uuid::Uuid::new_v4().to_string(); // Placeholder for project_id

                    db.add_secret(
                        name,
                        encrypted_value,
                        new_nonce,
                        fake_uuid.clone(),
                        "dev".to_string(),
                    )
                    .expect("Failed to create secret.");
                }
            }
        }
        Commands::List { project_id: _ } => {
            // In the future, we will list only secrets related to the project_id provided. For now, we will list all secrets.
            let secrets = db.get_all().expect("Failed to retrieve secrets.");
            if secrets.len() == 0 {
                println!("No secrets found.");
            } else {
                println!("Secrets:");
                for secret in secrets {
                    println!("- {}", secret.name);
                }
            }
        }
        Commands::Get { name: _ } => {
            // Implementation for get command
        }
        Commands::Delete { name: _ } => {
            // Implementation for delete command
        }
        Commands::Inject { commands: _ } => {
            // Implementation for inject command
        }
        Commands::Shell { shell: _ } => {
            // Implementation for shell command
        }
    }
}
