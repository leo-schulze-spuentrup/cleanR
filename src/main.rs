use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{self, BufWriter, Write};
use std::path::Path;
use std::process::{Command, Stdio};

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
    /// Run an activated routine once (used by the system scheduler)
    Run {
        /// Name of the routine
        #[arg(short, long)]
        name: String,
    },
    /// Run a routine immediately, regardless of activation state
    RunNow {
        /// Name of the routine
        #[arg(short, long)]
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
    #[serde(default)]
    target_directory: String,
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
        Commands::Run { name } => run_routine(name),
        Commands::RunNow { name } => run_routine_now(name),
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
                remove_active_routine_schedule(&path);
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
        remove_active_routine_schedule(Path::new(&path));
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

    let activating = routine.activated != "true";
    let message = if activating { "activated" } else { "deactivated" };
    if activating {
        if let Err(error) = set_crontab_job(name, Some(&routine.repetition)) {
            eprintln!("Failed to schedule routine '{}': {}", name, error);
            return;
        }
        routine.activated = "true".to_string();
    } else {
        if let Err(error) = set_crontab_job(name, None) {
            eprintln!("Failed to unschedule routine '{}': {}", name, error);
            return;
        }
        routine.activated = "false".to_string();
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
        let rollback = if activating {
            set_crontab_job(name, None)
        } else {
            set_crontab_job(name, Some(&routine.repetition))
        };
        if let Err(rollback_error) = rollback {
            eprintln!("Failed to roll back scheduler change: {}", rollback_error);
        }
    } else {
        println!("Routine is {}!", message);
    }
}

fn set_crontab_job(name: &str, schedule: Option<&str>) -> io::Result<()> {
    if let Some(schedule) = schedule {
        if schedule.trim().is_empty() || schedule.contains(['\n', '\r']) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "cron schedule must be a non-empty single line",
            ));
        }
    }

    let marker = cron_marker(name);
    let existing = Command::new("crontab").arg("-l").output()?;
    let existing_text = if existing.status.success() {
        String::from_utf8_lossy(&existing.stdout).into_owned()
    } else {
        let error = String::from_utf8_lossy(&existing.stderr);
        if error.to_lowercase().contains("no crontab") {
            String::new()
        } else {
            return Err(io::Error::other(error.into_owned()));
        }
    };

    let mut lines: Vec<String> = existing_text
        .lines()
        .filter(|line| !line.contains(&marker))
        .map(str::to_string)
        .collect();
    if let Some(schedule) = schedule {
        let working_directory = std::env::current_dir()?;
        let executable = std::env::current_exe()?;
        lines.push(format!(
            "{} cd {} && {} run --name {} {}",
            schedule.trim(),
            shell_quote(&working_directory.to_string_lossy()),
            shell_quote(&executable.to_string_lossy()),
            shell_quote(name),
            marker
        ));
    }

    let mut child = Command::new("crontab")
        .arg("-")
        .stdin(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let crontab = lines.join("\n");
    if let Some(stdin) = child.stdin.as_mut() {
        stdin.write_all(crontab.as_bytes())?;
        if !crontab.is_empty() {
            stdin.write_all(b"\n")?;
        }
    }
    drop(child.stdin.take());
    let output = child.wait_with_output()?;
    if output.status.success() {
        Ok(())
    } else {
        Err(io::Error::other(String::from_utf8_lossy(&output.stderr).into_owned()))
    }
}

fn cron_marker(name: &str) -> String {
    let encoded_name: String = name
        .as_bytes()
        .iter()
        .map(|byte| format!("{:02x}", byte))
        .collect();
    format!("# cleanR:{}", encoded_name)
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn remove_active_routine_schedule(path: &Path) {
    let content = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(_) => return,
    };
    let routine: Routine = match serde_json::from_str(&content) {
        Ok(routine) => routine,
        Err(_) => return,
    };
    if routine.activated == "true" {
        if let Err(error) = set_crontab_job(&routine.name, None) {
            eprintln!("Failed to remove schedule for '{}': {}", routine.name, error);
        }
    }
}

// Get User Input and return a Routine Struct
fn input_routine(name: &str) -> Routine {
    let name = name.to_string();
    let description = prompt("Enter description: ");
    let repetition = prompt("Cron schedule (five fields, e.g. 0 * * * *): ");
    let target_directory = prompt("Directory to scan: ");
    let whitelist = prompt_patterns("Whitelist globs, one per line; blank line to finish: ");
    let blacklist = prompt_patterns("Blacklist globs, one per line; blank line to finish: ");
    let exparation_date = prompt("Enter expiration date: ");
    let activated = "false".to_string(); // Default to inactive when created

    Routine {
        name,
        description,
        activated,
        repetition,
        target_directory,
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

fn prompt_patterns(message: &str) -> String {
    println!("{}", message);
    let mut patterns = Vec::new();
    loop {
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            eprintln!("Failed to read glob pattern.");
            continue;
        }
        let pattern = input.trim();
        if pattern.is_empty() {
            break;
        }
        if pattern == "quit" {
            std::process::exit(0);
        }
        patterns.push(pattern.to_string());
    }
    patterns.join("\n")
}

fn run_routine(name: &str) {
    execute_routine(name, true);
}

fn run_routine_now(name: &str) {
    execute_routine(name, false);
}

fn execute_routine(name: &str, require_activation: bool) {
    let file_path = format!("./routines/{}.json", name);
    let file_content = match fs::read_to_string(&file_path) {
        Ok(content) => content,
        Err(error) => {
            eprintln!("Failed to read routine '{}': {}", file_path, error);
            return;
        }
    };
    let routine: Routine = match serde_json::from_str(&file_content) {
        Ok(routine) => routine,
        Err(error) => {
            eprintln!("Failed to parse routine '{}': {}", file_path, error);
            return;
        }
    };

    if require_activation && routine.activated != "true" {
        eprintln!("Routine '{}' is not activated.", name);
        return;
    }
    if routine.target_directory.is_empty() {
        eprintln!("Routine '{}' has no target directory configured.", name);
        return;
    }

    let root = Path::new(&routine.target_directory);
    if !root.is_dir() {
        eprintln!("Routine target is not a directory: {}", root.display());
        return;
    }

    let whitelist_patterns: Vec<_> = routine.whitelist.lines().map(str::trim).filter(|line| !line.is_empty()).collect();
    if whitelist_patterns.is_empty() {
        eprintln!("Routine '{}' has no whitelist patterns; refusing to delete files.", name);
        return;
    }
    let whitelist = match build_globset(&routine.whitelist) {
        Ok(globs) => globs,
        Err(error) => {
            eprintln!("Invalid whitelist pattern in '{}': {}", name, error);
            return;
        }
    };
    let blacklist = match build_globset(&routine.blacklist) {
        Ok(globs) => globs,
        Err(error) => {
            eprintln!("Invalid blacklist pattern in '{}': {}", name, error);
            return;
        }
    };

    for entry in walkdir::WalkDir::new(root).follow_links(false).into_iter() {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                eprintln!("Failed to scan routine target: {}", error);
                continue;
            }
        };
        if !entry.file_type().is_file() {
            continue;
        }
        let relative_path = entry.path().strip_prefix(root).unwrap_or(entry.path());
        let relative_path = relative_path.to_string_lossy();
        if whitelist.is_match(relative_path.as_ref()) && !blacklist.is_match(relative_path.as_ref()) {
            match fs::remove_file(entry.path()) {
                Ok(()) => println!("Deleted: {}", entry.path().display()),
                Err(error) => eprintln!("Failed to delete {}: {}", entry.path().display(), error),
            }
        }
    }
}

fn build_globset(patterns: &str) -> Result<globset::GlobSet, globset::Error> {
    let mut builder = globset::GlobSetBuilder::new();
    for pattern in patterns.lines().map(str::trim).filter(|line| !line.is_empty()) {
        builder.add(globset::Glob::new(pattern)?);
    }
    builder.build()
}

fn modifyRoutine(attribute:&str, value:&str, ){

}

fn deletefile(){
    
}

#[cfg(test)]
mod tests {
    use super::build_globset;

    #[test]
    fn whitelist_matches_files_unless_blacklisted() {
        let whitelist = build_globset("**/*.log").unwrap();
        let blacklist = build_globset("**/important.log").unwrap();

        assert!(whitelist.is_match("logs/old.log"));
        assert!(!blacklist.is_match("logs/old.log"));
        assert!(whitelist.is_match("logs/important.log"));
        assert!(blacklist.is_match("logs/important.log"));
    }

    #[test]
    fn invalid_glob_patterns_are_rejected() {
        assert!(build_globset("[").is_err());
    }
}
