use std::io::{self, Write};

use base64::{Engine, engine::general_purpose::STANDARD};
use clap::Parser;
use crypto::gen_or_retrieve_key;
use rpassword::{ConfigBuilder, prompt_password, read_password, read_password_with_config};

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
    Set {
        name: String,
    },
    List {
        project_id: String,
    },
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
    // let DB_URL = format!("{}/.onyx/secrets.db", home_dir);
    // Dev
    let DB_URL = "./secrets.db".to_string();

    let cli = Cli::parse();
    let key = gen_or_retrieve_key().expect("Failed to generate or retrieve encryption key.");

    let db: database::Database = database::Database { url: DB_URL };

    match cli.command {
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
