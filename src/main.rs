use std::fs;
use std::fs::File;
use clap::{Parser, Subcommand};

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
        name: String

    },
    ///Delete one or more routines
    Delete {

        /// Name of the routine
        #[arg(short, long, default_value = "")]
        name: String,
        
        ///Removes all Routines
        #[arg(short, long)]
        all: bool,
    }
}

fn main() {
    // 3. Parse the arguments from the terminal
    let cli = Cli::parse();

    // 4. Match against the command to run your code
    match &cli.command{
        Commands::Create {name } => {
            createRoutine(name);
        }
        Commands::Delete {name,all} => {
            deleteRoutine(name, *all);
        }
    }
}

//Create Routine Function
fn createRoutine(name:&str){

    let routines_folder = "./routines";

    println!("A new routine named {} will be created at {}", name, routines_folder);
    println!("Configurationprocess started ...");
            
    let file_path = format!("{}/{}{}", routines_folder, name, ".json");
    print!("{}", file_path);

    let _file = File::create(file_path).expect("File couldnt be created. Make sure the Directory exists!");

    println!("Routine created!");
}

//Delete Routine Function
fn deleteRoutine(name:&str, all:bool){
    let routines_folder = "./routines";
    let routines = fs::read_dir("./routines").unwrap();

    //Logik für den all switch
    if all {
        for routine in routines {
            let entry = routine.unwrap();
            let path = entry.path();
        
            println!("Deleting: {}", path.display());
        
        if let Err(e) = fs::remove_file(&path) {
            eprintln!("Failed to delete {}: {}", path.display(), e);
        }
    }
    }

    //No name was put in 
    else{
        if(name == ""){
            println!("No Name Input, nothing was delted")
        }
        
        //Delete certain routines
        else {
            let path:String = format!("{}/{}{}", routines_folder, name, ".json");

            if let Err(e) = fs::remove_file(&path) {
                eprintln!("Failed to delete {}: {}", path, e);
            }
        }


    }
}