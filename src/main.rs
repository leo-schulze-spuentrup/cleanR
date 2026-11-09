//! cleaner - manage cleaning routines stored as JSON files.
//!
//! Commands: create, delete, modify.
//!
//! Storage: one JSON file per routine, `<id>.json`, inside the routines directory.
//! Directory resolution: `--dir` > `$CLEANER_HOME` > `~/.cleaner/routines`.
//!
//! Routines are addressed by numeric id or by name. An id always wins, so purely
//! numeric names are not allowed.

use clap::{Args, Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};
use std::{
    env, fs,
    io::{self, Write},
    path::{Path, PathBuf},
    process::ExitCode,
};

type Result<T> = std::result::Result<T, String>;

// ---------------------------------------------------------------------------
// Data model
// ---------------------------------------------------------------------------

/// A file is deleted when it satisfies ALL configured rules.
/// Unset rules are ignored.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
struct Rules {
    /// Last modification must be at least this many days ago.
    #[serde(skip_serializing_if = "Option::is_none")]
    older_than_days: Option<u64>,
    /// Minimum file size in bytes.
    #[serde(skip_serializing_if = "Option::is_none")]
    min_size: Option<u64>,
    /// Maximum file size in bytes.
    #[serde(skip_serializing_if = "Option::is_none")]
    max_size: Option<u64>,
    /// Allowed extensions (lowercase, without dot). Empty = any.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    extensions: Vec<String>,
    /// File name must contain this text (case-insensitive).
    #[serde(skip_serializing_if = "Option::is_none")]
    name_contains: Option<String>,
    /// Whether hidden files (leading '.') are eligible.
    include_hidden: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Routine {
    id: u32,
    name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    /// Directories this routine cleans.
    #[serde(default)]
    paths: Vec<PathBuf>,
    /// Descend into subdirectories.
    #[serde(default)]
    recursive: bool,
    #[serde(default)]
    rules: Rules,
}

impl Routine {
    fn has_rules(&self) -> bool {
        let r = &self.rules;
        r.older_than_days.is_some()
            || r.min_size.is_some()
            || r.max_size.is_some()
            || !r.extensions.is_empty()
            || r.name_contains.is_some()
    }

    fn print(&self) {
        println!("[{}] {}", self.id, self.name);
        if let Some(d) = &self.description {
            println!("  description : {d}");
        }
        println!("  recursive   : {}", self.recursive);
        println!("  paths       : {}", join_paths(&self.paths));
        let r = &self.rules;
        println!("  rules:");
        if let Some(v) = r.older_than_days {
            println!("    older than    : {v} days");
        }
        if let Some(v) = r.min_size {
            println!("    min size      : {}", fmt_size(v));
        }
        if let Some(v) = r.max_size {
            println!("    max size      : {}", fmt_size(v));
        }
        if !r.extensions.is_empty() {
            println!("    extensions    : {}", r.extensions.join(", "));
        }
        if let Some(v) = &r.name_contains {
            println!("    name contains : {v}");
        }
        println!("    include hidden: {}", r.include_hidden);
        if !self.has_rules() {
            println!("  warning: no matching rules set - every file would match!");
        }
    }
}

// ---------------------------------------------------------------------------
// CLI definition
// ---------------------------------------------------------------------------

#[derive(Parser)]
#[command(name = "cleaner", version, about = "Manage and run cleaning routines")]
struct Cli {
    /// Directory where routine JSON files are stored
    #[arg(long, global = true, value_name = "DIR")]
    dir: Option<PathBuf>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Create and configure a new, empty routine
    Create(CreateArgs),
    /// Delete one or multiple routines (by id or name)
    Delete(DeleteArgs),
    /// Modify an existing routine (by id or name)
    Modify(ModifyArgs),
}

/// Rule options shared by `create` and `modify`.
#[derive(Args)]
struct RuleArgs {
    /// Only files last modified at least N days ago
    #[arg(long, value_name = "DAYS")]
    older_than_days: Option<u64>,
    /// Only files at least this large (e.g. 500KB, 10MB, 1.5GB)
    #[arg(long, value_name = "SIZE", value_parser = parse_size)]
    min_size: Option<u64>,
    /// Only files at most this large
    #[arg(long, value_name = "SIZE", value_parser = parse_size)]
    max_size: Option<u64>,
    /// Only files whose name contains this text (case-insensitive)
    #[arg(long, value_name = "TEXT")]
    name_contains: Option<String>,
}

#[derive(Args)]
struct CreateArgs {
    /// Unique routine name
    name: String,
    /// Optional description
    #[arg(short, long)]
    description: Option<String>,
    /// Directory to clean (repeatable)
    #[arg(short, long = "path", value_name = "DIR")]
    paths: Vec<PathBuf>,
    /// Descend into subdirectories
    #[arg(short, long)]
    recursive: bool,
    /// Only match these extensions (repeatable, e.g. -e tmp -e log)
    #[arg(short = 'e', long = "ext", value_name = "EXT")]
    extensions: Vec<String>,
    /// Also match hidden files
    #[arg(long)]
    include_hidden: bool,
    #[command(flatten)]
    rules: RuleArgs,
}

#[derive(Args)]
struct DeleteArgs {
    /// One or more routine ids or names
    #[arg(required = true, value_name = "ID|NAME")]
    selectors: Vec<String>,
    /// Skip the confirmation prompt
    #[arg(short, long)]
    yes: bool,
}

#[derive(Clone, Copy, ValueEnum)]
enum ClearField {
    Description,
    OlderThan,
    MinSize,
    MaxSize,
    NameContains,
}

#[derive(Args)]
struct ModifyArgs {
    /// Routine id or name
    #[arg(value_name = "ID|NAME")]
    selector: String,
    /// Rename the routine
    #[arg(long, value_name = "NAME")]
    name: Option<String>,
    /// Set the description
    #[arg(short, long)]
    description: Option<String>,
    /// Add a directory (repeatable)
    #[arg(long, value_name = "DIR")]
    add_path: Vec<PathBuf>,
    /// Remove a directory (repeatable)
    #[arg(long, value_name = "DIR")]
    remove_path: Vec<PathBuf>,
    /// Enable/disable recursion
    #[arg(long, value_name = "BOOL")]
    recursive: Option<bool>,
    /// Add an extension (repeatable)
    #[arg(long, value_name = "EXT")]
    add_ext: Vec<String>,
    /// Remove an extension (repeatable)
    #[arg(long, value_name = "EXT")]
    remove_ext: Vec<String>,
    /// Enable/disable matching hidden files
    #[arg(long, value_name = "BOOL")]
    include_hidden: Option<bool>,
    /// Unset a field (repeatable)
    #[arg(long, value_enum, value_name = "FIELD")]
    clear: Vec<ClearField>,
    #[command(flatten)]
    rules: RuleArgs,
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

fn main() -> ExitCode {
    let cli = Cli::parse();
    let result = store_dir(cli.dir).and_then(|dir| match cli.command {
        Command::Create(a) => cmd_create(&dir, a),
        Command::Delete(a) => cmd_delete(&dir, a),
        Command::Modify(a) => cmd_modify(&dir, a),
    });
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

fn cmd_create(dir: &Path, a: CreateArgs) -> Result<()> {
    let routines = load_all(dir)?;
    validate_name(&a.name, &routines, None)?;

    let mut routine = Routine {
        id: routines.iter().map(|r| r.id).max().map_or(1, |m| m + 1),
        name: a.name.trim().to_string(),
        description: a.description,
        paths: Vec::new(),
        recursive: a.recursive,
        rules: Rules {
            older_than_days: a.rules.older_than_days,
            min_size: a.rules.min_size,
            max_size: a.rules.max_size,
            extensions: Vec::new(),
            name_contains: a.rules.name_contains,
            include_hidden: a.include_hidden,
        },
    };
    for p in a.paths {
        add_path(&mut routine.paths, p)?;
    }
    for e in a.extensions {
        add_ext(&mut routine.rules.extensions, &e);
    }
    validate_sizes(&routine.rules)?;

    save(dir, &routine)?;
    println!("Created routine:");
    routine.print();
    Ok(())
}

fn cmd_delete(dir: &Path, a: DeleteArgs) -> Result<()> {
    let routines = load_all(dir)?;

    // Resolve everything first so nothing is deleted if one selector is invalid.
    let mut targets: Vec<&Routine> = Vec::new();
    for sel in &a.selectors {
        let r = resolve(&routines, sel)?;
        if !targets.iter().any(|t| t.id == r.id) {
            targets.push(r);
        }
    }

    println!("Routines to delete:");
    for r in &targets {
        println!("  [{}] {}", r.id, r.name);
    }
    if !a.yes && !confirm(&format!("Delete {} routine(s)?", targets.len()))? {
        println!("Aborted.");
        return Ok(());
    }

    let mut failed = 0usize;
    for r in &targets {
        match fs::remove_file(routine_file(dir, r.id)) {
            Ok(()) => println!("Deleted [{}] {}", r.id, r.name),
            Err(e) => {
                failed += 1;
                eprintln!("Failed to delete [{}] {}: {e}", r.id, r.name);
            }
        }
    }
    if failed > 0 {
        return Err(format!("{failed} routine(s) could not be deleted"));
    }
    Ok(())
}

fn cmd_modify(dir: &Path, a: ModifyArgs) -> Result<()> {
    let routines = load_all(dir)?;
    let mut r = resolve(&routines, &a.selector)?.clone();

    if let Some(name) = a.name {
        validate_name(&name, &routines, Some(r.id))?;
        r.name = name.trim().to_string();
    }
    if let Some(d) = a.description {
        r.description = Some(d);
    }
    if let Some(v) = a.recursive {
        r.recursive = v;
    }
    if let Some(v) = a.include_hidden {
        r.rules.include_hidden = v;
    }
    for p in a.add_path {
        add_path(&mut r.paths, p)?;
    }
    for p in &a.remove_path {
        let before = r.paths.len();
        let canon = fs::canonicalize(p).ok();
        r.paths.retain(|x| x != p && Some(x) != canon.as_ref());
        if r.paths.len() == before {
            eprintln!("warning: path not in routine: {}", p.display());
        }
    }
    for e in &a.add_ext {
        add_ext(&mut r.rules.extensions, e);
    }
    for e in &a.remove_ext {
        let e = norm_ext(e);
        r.rules.extensions.retain(|x| *x != e);
    }

    // Set values first, then clears (so `--clear` wins if both are given).
    if let Some(v) = a.rules.older_than_days {
        r.rules.older_than_days = Some(v);
    }
    if let Some(v) = a.rules.min_size {
        r.rules.min_size = Some(v);
    }
    if let Some(v) = a.rules.max_size {
        r.rules.max_size = Some(v);
    }
    if let Some(v) = a.rules.name_contains {
        r.rules.name_contains = Some(v);
    }
    for f in a.clear {
        match f {
            ClearField::Description => r.description = None,
            ClearField::OlderThan => r.rules.older_than_days = None,
            ClearField::MinSize => r.rules.min_size = None,
            ClearField::MaxSize => r.rules.max_size = None,
            ClearField::NameContains => r.rules.name_contains = None,
        }
    }
    validate_sizes(&r.rules)?;

    save(dir, &r)?;
    println!("Modified routine:");
    r.print();
    Ok(())
}

// ---------------------------------------------------------------------------
// Validation & helpers
// ---------------------------------------------------------------------------

fn validate_name(name: &str, routines: &[Routine], except_id: Option<u32>) -> Result<()> {
    let name = name.trim();
    if name.is_empty() {
        return Err("name must not be empty".into());
    }
    if name.parse::<u32>().is_ok() {
        return Err("name must not be purely numeric (would clash with ids)".into());
    }
    if routines
        .iter()
        .any(|r| Some(r.id) != except_id && r.name.eq_ignore_ascii_case(name))
    {
        return Err(format!("a routine named '{name}' already exists"));
    }
    Ok(())
}

fn validate_sizes(r: &Rules) -> Result<()> {
    match (r.min_size, r.max_size) {
        (Some(min), Some(max)) if min > max => Err(format!(
            "min-size ({}) is larger than max-size ({})",
            fmt_size(min),
            fmt_size(max)
        )),
        _ => Ok(()),
    }
}

/// Resolve by id first, then by name (case-insensitive).
fn resolve<'a>(routines: &'a [Routine], sel: &str) -> Result<&'a Routine> {
    if let Ok(id) = sel.trim().parse::<u32>() {
        if let Some(r) = routines.iter().find(|r| r.id == id) {
            return Ok(r);
        }
    }
    routines
        .iter()
        .find(|r| r.name.eq_ignore_ascii_case(sel.trim()))
        .ok_or_else(|| format!("no routine found for '{sel}'"))
}

fn add_path(paths: &mut Vec<PathBuf>, p: PathBuf) -> Result<()> {
    let p = match fs::canonicalize(&p) {
        Ok(abs) => abs,
        Err(_) => {
            eprintln!("warning: path does not exist (yet): {}", p.display());
            p
        }
    };
    // Safety net: never allow a filesystem root as a cleaning target.
    if p.parent().is_none() && p.is_absolute() {
        return Err(format!("refusing to use filesystem root '{}'", p.display()));
    }
    if !paths.contains(&p) {
        paths.push(p);
    }
    Ok(())
}

fn norm_ext(e: &str) -> String {
    e.trim().trim_start_matches('.').to_ascii_lowercase()
}

fn add_ext(list: &mut Vec<String>, e: &str) {
    let e = norm_ext(e);
    if !e.is_empty() && !list.contains(&e) {
        list.push(e);
    }
}

fn join_paths(paths: &[PathBuf]) -> String {
    if paths.is_empty() {
        return "(none)".into();
    }
    paths
        .iter()
        .map(|p| p.display().to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

/// Parses sizes like `500`, `10KB`, `1.5GB` (units are powers of 1024).
fn parse_size(s: &str) -> std::result::Result<u64, String> {
    let s = s.trim();
    let split = s
        .find(|c: char| !(c.is_ascii_digit() || c == '.'))
        .unwrap_or(s.len());
    let (num, unit) = s.split_at(split);
    let n: f64 = num.parse().map_err(|_| format!("invalid size '{s}'"))?;
    let mult: f64 = match unit.trim().to_ascii_lowercase().as_str() {
        "" | "b" => 1.0,
        "k" | "kb" | "kib" => 1024.0,
        "m" | "mb" | "mib" => 1024.0 * 1024.0,
        "g" | "gb" | "gib" => 1024.0 * 1024.0 * 1024.0,
        "t" | "tb" | "tib" => 1024.0_f64.powi(4),
        other => return Err(format!("unknown size unit '{other}'")),
    };
    Ok((n * mult) as u64)
}

fn fmt_size(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut v = bytes as f64;
    let mut i = 0;
    while v >= 1024.0 && i < UNITS.len() - 1 {
        v /= 1024.0;
        i += 1;
    }
    if i == 0 {
        format!("{bytes} B")
    } else {
        format!("{v:.1} {}", UNITS[i])
    }
}

fn confirm(prompt: &str) -> Result<bool> {
    print!("{prompt} [y/N] ");
    io::stdout().flush().map_err(|e| e.to_string())?;
    let mut line = String::new();
    io::stdin().read_line(&mut line).map_err(|e| e.to_string())?;
    Ok(matches!(line.trim().to_ascii_lowercase().as_str(), "y" | "yes"))
}

// ---------------------------------------------------------------------------
// Storage
// ---------------------------------------------------------------------------

fn store_dir(cli_dir: Option<PathBuf>) -> Result<PathBuf> {
    let dir = if let Some(d) = cli_dir {
        d
    } else if let Some(d) = env::var_os("CLEANER_HOME") {
        PathBuf::from(d)
    } else {
        let home = env::var_os("HOME")
            .or_else(|| env::var_os("USERPROFILE"))
            .ok_or("cannot determine home directory; use --dir or set CLEANER_HOME")?;
        PathBuf::from(home).join(".cleaner").join("routines")
    };
    fs::create_dir_all(&dir).map_err(|e| format!("cannot create '{}': {e}", dir.display()))?;
    Ok(dir)
}

fn routine_file(dir: &Path, id: u32) -> PathBuf {
    dir.join(format!("{id}.json"))
}

fn load_all(dir: &Path) -> Result<Vec<Routine>> {
    let mut out = Vec::new();
    let entries =
        fs::read_dir(dir).map_err(|e| format!("cannot read '{}': {e}", dir.display()))?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let parsed = fs::read(&path)
            .map_err(|e| e.to_string())
            .and_then(|b| serde_json::from_slice::<Routine>(&b).map_err(|e| e.to_string()));
        match parsed {
            Ok(r) => out.push(r),
            Err(e) => eprintln!("warning: skipping '{}': {e}", path.display()),
        }
    }
    out.sort_by_key(|r| r.id);
    Ok(out)
}

/// Atomic save: write to a temp file, then rename over the target.
fn save(dir: &Path, r: &Routine) -> Result<()> {
    let target = routine_file(dir, r.id);
    let tmp = dir.join(format!("{}.json.tmp", r.id));
    let json = serde_json::to_vec_pretty(r).map_err(|e| e.to_string())?;
    fs::write(&tmp, json).map_err(|e| format!("cannot write '{}': {e}", tmp.display()))?;
    fs::rename(&tmp, &target).map_err(|e| format!("cannot save '{}': {e}", target.display()))
}