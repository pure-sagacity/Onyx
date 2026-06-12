use clap::Parser;

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
    let cli = Cli::parse();

    let database = database::Database {
        
    }

    match cli.command {
        Commands::Set { name: _ } => {
            // Implementation for set command
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
