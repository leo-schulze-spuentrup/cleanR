# cleanR
I want to create a opensource project to automaticlly and repeatetly delete or archive data on your PC. This should help to keep your workingenviroment organized while working on projects. The programm is written in Rust.

## Scheduled routines

On Linux, install the `crontab` utility to schedule routines. Creating a routine asks for a five-field cron schedule, a directory to scan, and whitelist and blacklist glob patterns (one pattern per line, ending each list with a blank line). A routine deletes regular files recursively when their relative paths match the whitelist and do not match the blacklist.

Use `cleanR activate --name <routine>` to register the routine in the current user's crontab. Run the same command again to deactivate it and remove its schedule. Cron invokes `cleanR run --name <routine>` at each scheduled time; this command does nothing unless the routine is still activated. Use `cleanR run-now --name <routine>` to execute it immediately, whether or not it is activated. Existing routine JSON files without a `target_directory` must be updated with a scan directory before they can run.
