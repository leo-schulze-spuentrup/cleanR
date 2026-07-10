use clap::{Parser, Subcommand};

// 1. The main entry point for your CLI tool
#[derive(Parser, Debug)]
#[command(name = "mycli", version, about = "My awesome custom CLI")]
struct Cli {
    // This tells clap to look for subcommands next
    #[command(subcommand)]
    command: Commands,
}

// 2. The enum containing all your available commands
#[derive(Subcommand, Debug)]
enum Commands {

    /// The actual implementation of your 'mycli' specific action
    Create {
        /// Name of the routine
        #[arg(short, long)]
        name: String,

        /// Place to create the routine
        #[arg(short, long, default_value = "./routines")]
        path: String,
    },
}

use std::fmt::format;
use std::fs::File;
use std::io::{BufWriter, Write};

fn main() {
    // 3. Parse the arguments from the terminal
    let cli = Cli::parse();

    // 4. Match against the command to run your code
    match &cli.command{
        Commands::Create { path, name } => {
            createRoutine(name, path);
        }
    }


}

fn createRoutine(name:&str, path:&str){
    println!("A new routine {} will be created at {}", name, path);
    println!("Configurationprocess started ...");
            
    let file_path = format!("{}/{}{}", path, name, ".json");
    print!("{}", file_path);

    let _file = File::create(file_path).expect("File couldnt be created. Make sure the Directory exists!");

    println!("Routine created!");
}