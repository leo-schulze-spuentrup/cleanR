# cleanR
I want to create a opensource project to automaticlly and repeatetly delete or archive data on your PC. This should help to keep your workingenviroment organized while working on projects. The programm is written in Rust.

## Scheduled routines

On Linux, install the `crontab` utility to schedule routines. Each routine uses a five-field cron schedule, a scan directory, and newline-separated whitelist and blacklist glob patterns. It recursively deletes regular files whose relative paths match the whitelist but not the blacklist.

## Commands

```
cleanR [--dir <DIR>] <COMMAND>
```

| Command  | Purpose                                              |
|----------|------------------------------------------------------|
| `create` | Create a new routine (or an empty one with `--empty`) |
| `modify` | Change an existing routine                           |
| `delete` | Delete one or more routines                          |
| `run`    | Run all routines and delete the files they match     |

Every command also accepts `-h`/`--help`. `cleanR -V`/`--version` prints the version.

### Global options

| Flag          | Description                                                                                       |
|---------------|---------------------------------------------------------------------------------------------------|
| `--dir <DIR>` | Directory where routine JSON files are stored. Default: `$CLEANR_HOME`, else `~/.cleanR/routines` |

### Selecting routines

Routines are identified by a numeric **id** or a unique **name** (case-insensitive). An id always takes priority, so purely numeric names are not allowed.

### Rules

A file is deleted only if it matches **all** rules set on the routine. Unset rules are ignored. A routine needs at least one path and one rule to be runnable.

| Rule                | Matches files that...                                   |
|---------------------|---------------------------------------------------------|
| `older_than_days`   | were last modified at least N days ago                  |
| `min_size`          | are at least this large                                 |
| `max_size`          | are at most this large                                  |
| `extensions`        | have one of the listed extensions (case-insensitive)    |
| `name_contains`     | have a file name containing the text (case-insensitive) |

Sizes accept plain bytes or a unit, with decimals allowed: `500`, `10KB`, `1.5GB`. Units are `B`, `K`/`KB`/`KiB`, `M`/`MB`/`MiB`, `G`/`GB`/`GiB`, `T`/`TB`/`TiB`, all powers of 1024.

---

### `create`

Create a new routine.

```
cleanR create <NAME> [OPTIONS]
cleanR create <NAME> --empty [-d <TEXT>]
```

A normal `create` requires at least one `--path` and at least one rule. `--empty` is the only way to create a routine without them.

| Argument / Flag                 | Description                                                                  |
|---------------------------------|------------------------------------------------------------------------------|
| `<NAME>`                        | Unique routine name (required, must not be purely numeric)                   |
| `-d`, `--description <TEXT>`    | Optional description                                                         |
| `--empty`                       | Create an empty routine (no paths, no rules). Cannot be combined with any option below |
| `-p`, `--path <DIR>`            | Directory to clean (repeatable)                                              |
| `-r`, `--recursive`             | Descend into subdirectories                                                  |
| `-e`, `--ext <EXT>`             | Only match this extension (repeatable, leading dot optional)                 |
| `--include-hidden`              | Also match hidden files (names starting with `.`)                            |
| `--older-than-days <DAYS>`      | Only files last modified at least N days ago                                 |
| `--min-size <SIZE>`             | Only files at least this large                                               |
| `--max-size <SIZE>`             | Only files at most this large                                                |
| `--name-contains <TEXT>`        | Only files whose name contains the text                                      |

Examples:

```
cleanR create "Old logs" -p ~/logs -r -e log -e tmp --older-than-days 30 --min-size 1MB
cleanR create "Downloads" --empty -d "Configure later"
```

---

### `modify`

Modify an existing routine. A modification can never leave a configured routine empty.

```
cleanR modify <ID|NAME> [OPTIONS]
```

| Argument / Flag                 | Description                                                                |
|---------------------------------|----------------------------------------------------------------------------|
| `<ID\|NAME>`                    | Routine to modify (required)                                               |
| `--name <NAME>`                 | Rename the routine                                                         |
| `-d`, `--description <TEXT>`    | Set the description                                                        |
| `--add-path <DIR>`              | Add a directory (repeatable)                                               |
| `--remove-path <DIR>`           | Remove a directory (repeatable)                                            |
| `--recursive <true\|false>`     | Enable or disable recursion                                                |
| `--add-ext <EXT>`               | Add an extension (repeatable)                                              |
| `--remove-ext <EXT>`            | Remove an extension (repeatable)                                           |
| `--include-hidden <true\|false>`| Enable or disable matching hidden files                                    |
| `--older-than-days <DAYS>`      | Set the age rule                                                           |
| `--min-size <SIZE>`             | Set the minimum size rule                                                  |
| `--max-size <SIZE>`             | Set the maximum size rule                                                  |
| `--name-contains <TEXT>`        | Set the name rule                                                          |
| `--clear <FIELD>`               | Unset a field (repeatable). Fields: `description`, `older-than`, `min-size`, `max-size`, `name-contains`. Wins over a set value given in the same command |

Examples:

```
cleanR modify "Old logs" --older-than-days 14 --add-ext bak --remove-ext tmp --clear min-size
cleanR modify 1 --name "Log cleanup" --recursive false
cleanR modify Downloads --add-path ~/Downloads --older-than-days 60 --add-ext iso
```

---

### `delete`

Delete one or more routines. All selectors are resolved first, so if one is unknown, nothing is deleted. This removes the routine definitions, not any cleaned files.

```
cleanR delete <ID|NAME>... [OPTIONS]
```

| Argument / Flag | Description                                  |
|-----------------|----------------------------------------------|
| `<ID\|NAME>...` | One or more routine ids or names (required)  |
| `-y`, `--yes`   | Skip the confirmation prompt                 |

Examples:

```
cleanR delete 1 "Temp files"
cleanR delete 2 3 --yes
```

---

### `run`

Run **all** stored routines and delete the files they match. There is no confirmation prompt, so it is safe to use in scheduled jobs.

```
cleanR run [OPTIONS]
```

| Flag              | Description                                                     |
|-------------------|-----------------------------------------------------------------|
| `-n`, `--dry-run` | List what would be deleted and how much space it would free, without deleting anything |

Behavior:

- Routines without at least one path and one rule are skipped with a warning.
- Only regular files are deleted. Symlinks are never followed or removed, and directories are never removed.
- Hidden files and hidden directories are skipped unless the routine has `include_hidden` enabled.
- Subdirectories are only searched if the routine is `recursive`.
- Errors (unreadable directories, failed deletions) are reported without stopping the run. The exit code is non-zero if any occurred.

Examples:

```
cleanR run --dry-run
cleanR run
```