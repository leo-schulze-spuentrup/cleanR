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
        name: String,
        

        /// Create a filter
        #[arg(short, long)]
        filter: bool



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

//Defining needed Structs to generate routines and filters
struct Routine{
    
}

struct Filter{

}


fn main() {
    // 3. Parse the arguments from the terminal
    let cli = Cli::parse();

    // 4. Match against the command to run your code
    match &cli.command{
        Commands::Create {name, filter } => {
            createRoutine(name, *filter);
        }
        Commands::Delete {name,all} => {
            deleteRoutine(name, *all);
        }
    }
}

//Create Routine Function
fn createRoutine(name:&str, filter:bool){

    if(filter){

        let filter_folder = "./filters";

        println!("A new filter named {} will be created at {}", name, filter_folder);
        println!("Configurationprocess started ...");
            
        let file_path = format!("{}/{}{}", filter_folder, name, ".json");
        print!("{}", file_path);

        let _file = File::create(file_path).expect("File couldnt be created. Make sure the Directory exists!");

        println!("Routine created!");
    }
    else{
        let routines_folder = "./routines";

        println!("A new routine named {} will be created at {}", name, routines_folder);
        println!("Configurationprocess started ...");
            
        let file_path = format!("{}/{}{}", routines_folder, name, ".json");
        print!("{}", file_path);

        let _file = File::create(file_path).expect("File couldnt be created. Make sure the Directory exists!");

        println!("Routine created!");
    }
}

//Delete Routine Function
fn deleteRoutine(name:&str, all:bool){
    let routines_folder = "./routines";
    let routines = fs::read_dir(routines_folder).unwrap();
    
    //Clearing the whole cleanR-data
    if all {
        for routine in routines {
            let entry = routine.unwrap();
            let path = entry.path();
        
            println!("Deleting: {}", path.display());
        
        if let Err(e) = fs::remove_file(&path) {
            eprintln!("Failed to delete routine {}: {}", path.display(), e);
        }
    }
        let filter_folder = "./filters";
        let filters = fs::read_dir(filter_folder).unwrap();

        for filter in filters {
            let entry = filter.unwrap();
            let path = entry.path();
        
            println!("Deleting: {}", path.display());
        
        if let Err(e) = fs::remove_file(&path) {
            eprintln!("Failed to delete routine {}: {}", path.display(), e);
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