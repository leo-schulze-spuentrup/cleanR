use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{self, BufWriter, Write};

// 1. The main entry point for your CLI tool
#[derive(Parser, Debug)]
#[command(name = "cleanR", version, about = "keep your system clean")]
struct Cli {
    // This tells clap to look for subcommands next
    #[command(subcommand)]
    command: Commands,
}

// 2. The enum containing all your available commands
#[derive(Subcommand, Debug)]
enum Commands {
    /// Creates a routine
    Create {
        /// Name of the routine
        #[arg(short, long)]
        name: String,

        /// Create a filter
        #[arg(short, long)]
        filter: bool,
    },
    /// Delete one or more routines
    Delete {
        /// Name of the routine
        #[arg(short, long, default_value = "")]
        name: String,

        /// Removes all Routines
        #[arg(short, long)]
        all: bool,
    },
    /// Used to activate a Routine
    Activate {
        /// Name of the routine
        #[arg(short, long, default_value = "")]
        name: String,
    },
}

// Defining needed Structs to generate routines and filters
// Added Deserialize so serde can read JSON back into this struct
#[derive(Serialize, Deserialize, Debug)]
struct Routine {
    name: String,
    description: String,
    activated: String,
    repetition: String,
    whitelist: String,
    blacklist: String,
    exparation_date: String,
}

fn main() {
    // 3. Parse the arguments from the terminal
    let cli = Cli::parse();

    // 4. Match against the command to run your code
    match &cli.command {
        Commands::Create { name, filter } => {
            create_routine(name, *filter);
        }
        Commands::Delete { name, all } => {
            delete_routine(name, *all);
        }
        Commands::Activate { name } => {
            if name.is_empty() {
                println!("Please provide a routine name using --name or -n");
            } else {
                activate_routine(name);
            }
        }
    }
}

// Create Routine Function
fn create_routine(name: &str, filter: bool) {
    if filter {
        let filter_folder = "./filters";

        println!("A new filter named {} will be created at {}", name, filter_folder);
        println!("Configuration process started ...");

        let file_path = format!("{}/{}.json", filter_folder, name);
        println!("{}", file_path);

        let _file = File::create(&file_path)
            .expect("File couldn't be created. Make sure the directory exists!");

        println!("Filter created!");
    } else {
        let routines_folder = "./routines";

        println!("A new routine named {} will be created at {}", name, routines_folder);
        println!("Configuration process started ...");

        let file_path = format!("{}/{}.json", routines_folder, name);
        println!("{}", file_path);

        let file = File::create(&file_path)
            .expect("File couldn't be created. Make sure the directory exists!");

        let writer = BufWriter::new(file);

        let routine: Routine = input_routine(name);

        serde_json::to_writer_pretty(writer, &routine)
            .expect("Failed to write JSON to file");

        println!("Routine created!");
    }
}

// Delete Routine Function
fn delete_routine(name: &str, all: bool) {
    let routines_folder = "./routines";

    // Clearing all data
    if all {
        if let Ok(routines) = fs::read_dir(routines_folder) {
            for routine in routines.flatten() {
                let path = routine.path();
                println!("Deleting: {}", path.display());
                if let Err(e) = fs::remove_file(&path) {
                    eprintln!("Failed to delete routine {}: {}", path.display(), e);
                }
            }
        }

        let filter_folder = "./filters";
        if let Ok(filters) = fs::read_dir(filter_folder) {
            for filter in filters.flatten() {
                let path = filter.path();
                println!("Deleting: {}", path.display());
                if let Err(e) = fs::remove_file(&path) {
                    eprintln!("Failed to delete filter {}: {}", path.display(), e);
                }
            }
        }
    } else if name.is_empty() {
        println!("No name input provided, nothing was deleted.");
    } else {
        let path = format!("{}/{}.json", routines_folder, name);
        if let Err(e) = fs::remove_file(&path) {
            eprintln!("Failed to delete {}: {}", path, e);
        } else {
            println!("Deleted: {}", path);
        }
    }
}

// Activate Routine Function (Reads JSON, modifies `activated`, writes back)
fn activate_routine(name: &str) {
    let routines_folder = "./routines";
    let file_path = format!("{}/{}.json", routines_folder, name);

    // 1. Read existing file content
    let file_content = match fs::read_to_string(&file_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Failed to read routine at '{}': {}", file_path, e);
            return;
        }
    };

    // 2. Parse JSON into the Routine struct
    let mut routine: Routine = match serde_json::from_str(&file_content) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Failed to parse JSON in '{}': {}", file_path, e);
            return;
        }
    };

    // 3. Update the activated field
    let mut message: &str = "";
    if(routine.activated == "" || routine.activated == "false"){
        routine.activated = "true".to_string();
        message = "activated"
    }
    else {
        routine.activated = "false".to_string();
        message = "deactivated"
    }

    // 4. Serialize back to pretty JSON
    let updated_json = match serde_json::to_string_pretty(&routine) {
        Ok(json) => json,
        Err(e) => {
            eprintln!("Failed to serialize updated routine: {}", e);
            return;
        }
    };

    // 5. Overwrite the existing file
    if let Err(e) = fs::write(&file_path, updated_json) {
        eprintln!("Failed to write updated routine to '{}': {}", file_path, e);
    } else {
        println!("Routine is {}!", message);
    }
}

// Get User Input and return a Routine Struct
fn input_routine(name: &str) -> Routine {
    let name = name.to_string();
    let description = prompt("Enter description: ");
    let repetition = prompt("Time of repetition: ");
    let whitelist = prompt("Enter deletion criteria: ");
    let blacklist = prompt("Enter keeping criteria: ");
    let exparation_date = prompt("Enter expiration date: ");
    let activated = "false".to_string(); // Default to inactive when created

    Routine {
        name,
        description,
        activated,
        repetition,
        whitelist,
        blacklist,
        exparation_date,
    }
}

fn prompt(message: &str) -> String {
    let mut input = String::new();

    loop {
        print!("{}", message);
        let _ = io::stdout().flush();

        input.clear();
        if io::stdin().read_line(&mut input).is_ok() {
            let trimmed = input.trim();
            if !trimmed.is_empty() {
                if trimmed == "quit" {
                    std::process::exit(0);
                }
                return trimmed.to_string();
            }
        }

        println!("Input cannot be empty. Exit: quit");
    }
}

fn modifyRoutine(attribute:&str, value:&str, ){

}
