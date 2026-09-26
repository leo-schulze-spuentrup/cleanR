# cleanR
I want to create a opensource project to automaticlly and repeatetly delete or archive data on your PC. This should help to keep your workingenviroment organized while working on projects. The programm is written in Rust.

## Scheduled routines

On Linux, install the `crontab` utility to schedule routines. Each routine uses a five-field cron schedule, a scan directory, and newline-separated whitelist and blacklist glob patterns. It recursively deletes regular files whose relative paths match the whitelist but not the blacklist.

Commands:
- `cleanR activate --name <routine>`: Activate and schedule a routine; run again to deactivate and remove its schedule.
- `cleanR run --name <routine>`: Run a scheduled routine once, only if it is activated. Cron invokes this command.
- `cleanR run-now --name <routine>`: Run a routine immediately, whether or not it is activated.

Existing routine JSON files without a `target_directory` need a scan directory before they can run.
