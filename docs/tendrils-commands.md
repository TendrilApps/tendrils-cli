# Tendrils Commands
> **Warning:**  
> Tendrils is still quite new and these commands are subject to change. Expect any scripts relying on them to break in the future.

- `td` is the CLI tool that performs these commands
- List all commands using the `--help (-h)` flag
``` bash
td --help (-h)
```

# Initializing a Tendrils Folder
- Creates a starter [`tendrils.json`](./configuration.md#tendrilsjson) file in the current folder or in a given path
- It's recommended to do this in an empty folder
``` bash
td init
```

## Forced Init Modifier
- Ignores errors due to a non-empty folder
- Uses the `--force (-f)` flag
```bash
td init --force (-f)
```

# Listing Tendrils
- Lists extended information about the tendrils
``` bash
td list
```

# Tendril Actions
- There are several actions for working with tendrils 
- `td` is the CLI tool that performs these commands
- Each action must be called from or pointed to a [Tendrils repo](../README.md#tendrils-repo)
    - See [Specifying the Tendrils Repo](#specifying-the-tendrils-repo)

## Pulling
- Copies tendrils from their locations on the computer to the [Tendrils repo](../README.md#tendrils-repo)
- Only operates on [copy-type](../README.md#copy-type-tendrils) tendrils
- Only the *first* [remote](./configuration.md#remotes) is used

```bash
td pull
```

## Pushing
- Copies tendrils from the Tendrils folder to their various locations on the machine
- Only operates on [copy-type](../README.md#copy-type-tendrils) tendrils
- *Each* [remote](./configuration.md#remotes) is used
```bash
td push
```

## Linking
- Creates symlinks at the various locations on the computer to the tendrils in the [Tendrils repo](../README.md#tendrils-repo)
- Only operates on [link-type](../README.md#link-type-tendrils) tendrils
- *Each* [remote](./configuration.md#remotes) is used
``` bash
td link
```

## "Out" Action
- Performs all outward bound actions
- Will [link](#linking) all [link-type](../README.md#link-type-tendrils) tendrils
- Will [push](#pushing) all [copy-type](../README.md#copy-type-tendrils) tendrils
``` bash
td out
```

## Dry Run Modifier
- Uses the `--dry-run (-d)` flag
- Available on all of the actions listed above
- Will perform the internal checks for the action but does not modify anything on the file system. If the action is expected to fail, the expected error is displayed. If it's expected to succeed, it displays as `Skipped`. Note: It is still possible for a successful dry run to fail in an actual run.
- If this flag is not included, the action will modify the file system as normal
``` bash
td push --dry-run (-d)
```

## Forced Run Modifier
- Uses the `--force (-f)` flag
- Available on all of the actions listed above
- Will ignore any type mismatches and will force the operation
- If this flag is not included, the action will display an error for any type mismatches
- Type mismatches occur when the source and destination file system objects do not match, or do not match the expected types, such as:
    - The source is a file but the destination is a folder
    - The local or remote are symlinks (during a push/pull action)
    - The remote is *not* a symlink (during a link action)
``` bash
td push --force (-f)
```

# Common Command Options
- These options are available on several of the commands listed above

## Specifying the Tendrils repo
- A path can be explicitly set using the `--path` argument
    - Available on all of the actions listed above
    - In general, the [path resolving](./configuration.md#path-resolving) rules will be applied, with the exception of:
        - Relative paths will be appended to the *current working directory* instead of appending it to `/` or `\`
``` bash
td push --path /some/tendrils/folder
```

- If no `--path` argument is provided:
    1. Tendrils will first check if the current working directory is a [Tendrils repo](../README.md#tendrils-repo). If it is, this folder (and the tendrils defined in its [`tendrils.json`](./configuration.md#tendrilsjson)) will be used for the command
    2. If the CWD is not a Tendrils folder, then the [default repo](./configuration.md#default-repo-path) will be checked

## Filtering Tendrils
- For any of the commands that operate on a set of tendrils, the given tendrils can be specified further using the filters below
- These filters are cumulative
- For the filters below that support glob patterns, these are resolved using the [`glob-match`](https://crates.io/crates/glob-match) crate
    - Consult this crate's documentation for the syntax

![](../assets/profiles-demo.gif)

### Filtering by Locals
- Using the `--locals (-l)` argument
- Available on all of the actions listed above
- Only tendrils who's [local](./configuration.md#local-path) matches any of the given filters will be included
    - Glob patterns are supported
``` bash
td link -l file1.txt SomeFolder/file2.txt **/*.json
```
- Will only include tendrils whose local path is exactly `file1.txt` or `SomeFolder/file2.txt`, and all JSON files
- Note: Local paths are filtered *before* appending to the [repo](../README.md#tendrils-repo) path

### Filtering by Remotes
- Using the `--remotes (-r)` argument
- Available on all of the actions listed above
- Only includes tendril [remotes](./configuration.md#remotes) that match any of the given remotes
    - Glob patterns are supported
- Any tendril remotes that do not match are omitted, and any tendrils without any matching remotes are omitted entirely.
- Note: Remotes are filtered *before* they are [resolved](./configuration.md#path-resolving)
``` bash
td push -p ~/Library/SomeApp/config.json **/*OneDrive*/**
```
- Will only include tendrils whose remote is exactly `~/Library/SomeApp/config.json`, or any path that contains `OneDrive`

### Filtering by Profile
- Using the `--profiles (-P)` argument
- Available on all of the actions listed above
- Only tendrils with one or more matching [profiles](./configuration.md#profiles) will be included
    - Glob patterns are supported
- Tendrils without any profiles specified will still be included
``` bash
td push -P home mac
```
- Will include any tendrils with the `home` or `mac` profile, and any that don't have a profile
- When this argument is not provided, the [default profiles](./configuration.md#default-profiles) are used
