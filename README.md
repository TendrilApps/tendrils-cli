<div align="center" >
    <img src="./assets/logo.svg" width="200" />
</div>

# General
- Tendrils is a tool to manage specific files/folders strewn about the computer from a single location
    - Each file/folder is defined by a tendril in the [`tendrils.json`](#tendrilsjson) file
    - They are all stored in a common [Tendrils repo](#tendrils-repo)
    - `td` is the CLI tool to manage these tendrils
- Main uses include:
    - Versioning/syncing configuration files that are expected to be in specific locations on the machine (video game saves, application settings, `.bashrc`, `.vim`, etc)
    - Versioning/syncing small scripts that otherwise would not have their own repos
    - Quickly editing miscellaneous files in a common place rather than tracking them down individually
    - Multiple settings profiles within one user
- Where Tendrils isn't as useful:
    - Ephemeral environments, such as Docker containers. It's likely simpler to use a one-off script/Dockerfile to put the files where they belong

# License & Acknowledgements
- GPLv3 - See additional licensing information in the [license file](./LICENSE.md)
- This project uses several open source libraries, and their licensing information can be found in the [third party license file](./LICENSE-3RD-PARTY.md)
- License and other info can be displayed using the `td about` command
``` bash
td about
```

# Getting Started
## Supported Platforms
- Mac
- Windows

## Installation
- Currently, Tendrils is only distributed through source code and must be built using the Cargo toolchain
- To build the `td` CLI:
``` bash
# From the 'tendrils' workspace folder
cargo build           # For a 'debug' build
cargo build --release # For a 'release' build
```
- By default, the output executable is placed in `target/debug` or `target/release` depending which profile was used to build it
- Once built, it is recommended to update your `PATH` variable to include this location (or copy the executable to somewhere in your `PATH`)

## Set Up
1. Create a new empty folder that will become the [Tendrils repo](#tendrils-repo)
2. `cd` into the new folder, then run the `td init` command
    -  This will create a starter [`tendrils.json`](#tendrilsjson) file inside the new folder
``` bash
cd MyTendrilsFolder
td init
```

3. Define some tendrils in the file following the [schema](#tendrilsjson-schema)
4. Run a [`pull`](#pulling) to make an initial copy of any [push/pull style](#pushpull-style) tendrils to the [Tendrils repo](#tendrils-repo)
5. Run a [`link`](#linking) to setup any [link style](#link-style) tendrils

# Tendrils Repo
- Serves as a common location for all of the tendrils defined in the [`tendrils.json`](#tendrilsjson) file
- The master copies are stored here
- Any folder with a `.tendrils` subfolder containing a [`tendrils.json`](#tendrilsjson) file is considered a Tendrils repo
    - Similar to how a Git repo has a `.git` folder at its top level
- Items are structured according to their [`local`](#tendrilsjson-schema) path

## Version Control & Synchronization
- Tendrils itself does not have versioning or synchronization functionality, but the Tendrils repo is often placed inside a synchronized folder such as OneDrive, or under a version control system like Git

# Tendril Styles
## Push/Pull Style
- These tendrils rely on copying back and forth between the various locations on the computer and the [Tendrils repo](#tendrils-repo)
- Designated by setting [`link`](#link) to `false`

## Link Style
- These tendrils are setup as symlinks rather than being copied back and forth
- The symlinks are created at the various locations on the computer and all target the same file/folder in the [Tendrils repo](#tendrils-repo)
- Designated by setting [`link`](#link) to `true`

# `tendrils.json`
- Specifies all of the files and directories to be considered as tendrils
- Stored in the `.tendrils` folder inside a [Tendrils repo](#tendrils-repo)

## `tendrils.json` Schema
- The json schema is intended to be flexible and to allow defining multiple tendrils in a compact form

TODO: To be updated once schema is updated
```json
{
    "tendrils": [
        {
            "group": "Group/app Name",
            "names": [
                "file1.txt",
                "file2.md",
                "folder1"
            ],
            "parents": [
                "~/path/to/parent/folder",
                "parent/with/<ENV_VAR>",
            ],
            "dir-merge": false,
            "link": false,
            "profiles": [
                "home",
                "work",
            ]
        }
    ]
}
```
- The example above would define 6 tendrils
    - One for each of the 3 names in each of the 2 parent folders
    - The profiles/tendril modes are applied to all of these tendrils
- `null` is not valid in any of these fields
- Must only contain valid UTF-8 characters

### `local`
- The path to the master copy of the file/folder inside the [Tendrils Repo](#tendrils-repo)
    - This path is appended to the Tendrils repo
- If the path specifies multiple subfolders, the corresponding folder structure will be created
    - These paths should use `/` instead of `\` due to better cross-platform support
- Variables/other values in `local` are *not* [resolved](#path-resolving), but directory separators are [replaced](#directory-separators) on Windows
- The `local` should not:
    - Be whitespace only
    - Point to the repo itself
    - Point to anything outside of the repo
    - Have path components containing only `..`
    - Point to the `.tendrils` folder or anything inside of it
- An attempt is made to detect values that break the guidelines above, but there are several edge cases that may not be detected
- See [Filtering by Local](#filtering-by-locals)
```json
# These paths are relative to the root of the Tendrils repo
"local": "file.txt" # Becomes /path/to/tendrils/repo/file.txt

# Or
"local": "SomeFolder" # Becomes /path/to/tendrils/repo/SomeFolder/file.txt

# Or
"local": "SomeFolder/file.txt" # Becomes /path/to/tendrils/repo/SomeFolder/file.txt
```

### `remotes`
- A list of paths to the files/folders throughout the host machine that are associated with the corresponding [`local`](#local)
- Each remote defines a tendril
- Environment variables and some path abbreviations are supported
    - See [Path Resolving](#path-resolving)
- If the list is empty, no tendrils are defined
- If there is only one remote, the square brackets can be omitted
``` json
"remotes": "~/some/specific/location/file.txt"
# Or
"remotes": ["~/some/specific/location/SomeFolder"]
# Or
"remotes": ["~/some/specific/location/file.txt", "~/another/specific/location/file.txt"]
```
- Remotes should be absolute paths
- Cross-platform paths should use `/` instead of `\` due to [the handling of directory separators](#directory-separators)
- Care must be taken to avoid [recursive tendrils](#recursive-tendrils)

### `dir-merge`
- Specifies the merge strategy when folders are copied to or from the [Tendrils repo](#tendrils-repo)
- `true` - Add any new files, overwrite any conflicting files, but do not delete any files already in the destination folder
- `false` - Entirely replace the destination folder with the source folder
- If this field is omitted, it defaults to `false`
- This setting has no effect on the behaviour of file tendrils or link style tendrils
    - It is only relevant for [push/pull style](#pushpull-style) folder tendrils
- Note: this field may be overriden depending on the value of [`link`](#link)

### `link`
- `true` - Designates these tendrils as [link style](#link-style)
    - This overrides any setting for [`dir-merge`](#dir-merge)
- `false` - Designates these tendrils as [push/pull style](#pushpull-style)
- If this field is omitted, it defaults to `false`

### `profiles`
- Provide an additional means for associating groups of tendrils (in a broader sense than `group`)
    - They may group by context, and often map to a specific computer (`home`, `work`, etc), or to a group of computers (`unix`, `windows`, etc)
    - They may also be used to group by category (`configs`, `scripts`, etc.)
- A tendril can belong to several profiles at once
- If no profiles are specified, these tendrils are always included regardless of which profiles are [specified](#filtering-by-profile) in the command
- If this field is omitted, it defaults to an empty list
- If there is only one profile, the square brackets can be omitted
```json
"profiles": "my-profile"
# Or
"profiles": ["my-profile"]
# Or
"profiles": ["my-profile1", "my-profile2"]
```

# Tendril Actions
- There are several actions for working with tendrils 
- `td` is the CLI tool that performs these commands
- Each action must be called from or pointed to a [Tendrils repo](#tendrils-repo)
    - See [specifying a Tendrils repo](#specifying-the-tendrils-repo)

## Pulling
- Copies tendrils from their locations on the computer to the [Tendrils repo](#tendrils-repo)
- Only operates on [push/pull style](#pushpull-style) tendrils
- Only the *first* [remote](#remotes) is used

```bash
td pull
```

## Pushing
- Copies tendrils from the Tendrils folder to their various locations on the machine
- Only operates on [push/pull style](#pushpull-style) tendrils
- *Each* [remote](#remotes) is used
```bash
td push
```

## Linking
- Creates symlinks at the various locations on the computer to the tendrils in the [Tendrils repo](#tendrils-repo)
- Only operates on [link style](#link-style) tendrils
- *Each* [remote](#remotes) is used
``` bash
td link
```

## "Out" Action
- Performs all outward bound actions
- Will [link](#linking) all [link style](#link-style) tendrils
- Will [push](#pushing) all [push/style style](#pushpull-style) tendrils
``` bash
td out
```

## Dry Run Modifier
- Uses the `--dry-run (-d)` flag
- Available on all of the actions listed above
- Will perform the internal checks for the action but does not modify anything on the file system. If the action is expected to fail, the expected error is displayed. If it's expected to succeed, it displays as `Skipped`. Note: It is still possible for a successful dry run to fail in an actual run.
- If this flag is not included, the action will modify the file system as normal, and will display `Ok` if successful
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

## Specifying the Tendrils repo
- If no `--path` argument is provided:
    1. Tendrils will first check if the current working directory is a [Tendrils repo](#tendrils-repo). If it is, this folder (and the tendrils defined in its [`tendrils.json`](#tendrilsjson)) will be used for the command
    2. If the CWD is not a Tendrils folder, then the [default repo](#default-repo-path) will be checked
- A path can be explicitly set using the `--path` argument
    - Available on all of the actions listed above
    - In general, the [path resolving](#path-resolving) rules will be applied, with the exception of:
        - Relative paths will be appended to the *current working directory* instead of appending it to `/` or `\`
``` bash
td push --path /some/tendrils/folder
```

## Filtering Tendrils
- These filters are cumulative
- For the filters below that support glob patterns, these are resolved using the [`glob-match`](https://crates.io/crates/glob-match) crate
    - Consult this crate's documentation for the syntax

### Filtering by Locals
- Using the `--locals (-l)` argument
- Available on all of the actions listed above
- Only tendrils who's [local](#local) matches any of the given filters will be included
    - Glob patterns are supported
``` bash
td link -l file1.txt SomeFolder/file2.txt **/*.json
```
- Will only include tendrils whose local path is exactly `file1.txt` or `SomeFolder/file2.txt`, and all JSON files

### Filtering by Remotes
- Using the `--remotes (-r)` argument
- Available on all of the actions listed above
- Only includes tendril [remotes](#remotes) that match any of the given remotes
    - Glob patterns are supported
- Any tendril remotes that do not match are omitted, and any tendrils without any matching remotes are omitted entirely.
- Note: Remotes are filtered *before* they are [resolved](#path-resolving)
``` bash
td push -p ~/Library/SomeApp/config.json **/*OneDrive*/**
```
- Will only include tendrils whose remote is exactly `~/Library/SomeApp/config.json`, or any path that contains `OneDrive`

### Filtering by Profile
- Using the `--profiles (-P)` argument
- Available on all of the actions listed above
- Only tendrils with one or more matching [profiles](#profiles) will be included
    - Glob patterns are supported
- Tendrils without any profiles specified will still be included
``` bash
td push -P home mac
```
- Will include any tendrils with the `home` or `mac` profile, and any that don't have a profile

# Path Resolving
- Paths will be resolved in the following order:
    1. Environment variables [are resolved](#resolving-environment-variables)
    2. A leading tilde (`~`) [is resolved](#resolving-tilde-)
    3. Relative paths are [converted to absolute](#relative-paths)
    4. Directory separators [are replaced](#directory-separators)

## Resolving Environment Variables
- A [remote path](#remotes) or [repo path](#specifying-the-tendrils-repo) containing environment variables in the form `<VAR_NAME>` will be replaced with the variable values
``` json
"remotes": "<APPDATA>\\Local\\SomeApp\\file.txt"
```
- The above example will resolve to `C:\Users\YourUsername\AppData\Local\SomeApp\\file.txt` on a typical Windows installation
``` bash
td push --path <OneDrive>/MyRepo
```
- A path can contain multiple environment variables

## Resolving Tilde (`~`)
- A [remote path](#remotes) or a [repo path](#specifying-the-tendrils-repo) with a leading tilde will replace the `~` with the value of the `HOME` environment variable
    - If `HOME` is not set, it will fall back to the combination of `HOMEDRIVE` and `HOMEPATH`
    - If either of those are not set, the raw path is used
``` json
"remotes": "~/documents/file.txt"
```
- The above example will resolve to `your/home/path/documents/file.txt`
``` bash
td push --path ~/MyRepo
```

## Directory Separators
- Forward slashes (`/`) are replaced by backslashes (`\`) on Windows only
- Backslashes are not changed on Unix
- If a path is intended for cross-platform use, it should be specified using `/` for all directory separatators

## Relative Paths
- If the path is still relative after [resolving variables](#resolving-environment-variables) and [tilde](#resolving-tilde-), it will be converted to absolute by prepending `/` on Unix or `\` on Windows
- What counts as an absolute path varies by platform
    - `C:\Path` and `\\MyServer\Share\Path` are absolute on Windows but not on Unix. On Unix, these would be converted to `/C:\Path` and `/\\MyServer\Share\Path`
- This blind conversion can cause some unintuitive behaviour on Windows
    - For example `C:` or `C:SomePath` are converted to `\C:` and `\C:SomePath` rather than the `C:\` and `C:\SomePath` that may have been expected
- `.` or `..` components are not modified and are left to the OS to resolve
    - For example `/Users/MyUser/./Desktop/../Downloads` would be passed as-is to the OS, and the OS should resolve this to `/Users/MyUser/Downloads`

## Other URL Types
- Other URL types such as `file:///` and `https://` are not supported

## Recursive Tendrils
- Recursive tendrils can cause [actions](#tendril-actions) to fail, or can cause unintuitive copying/linking behaviour
- Occurs in any of the following cases:
    - The remote path is a directory that contains the Tendrils repo
    - The remote path is the Tendrils repo
    - The remote path is a file or directory inside the Tendrils repo
- An attempt is made to detect these cases, but due to the vast number of ways to specify a path, there are several cases in which these checks can fail
    - *The user must take special care to prevent specifying a recursive tendril*

| Repo Path | Recursive Remote Path |
| --- | --- |
| `/Users/MyUser/MyRepo` | `/Users/MyUser` |
| `/Users/MyUser/MyRepo` | `/Users/MyUser/MyRepo` |
| `/Users/MyUser/MyRepo` | `/Users/MyUser/MyRepo/misc.txt` |

# Configuration
- Global configuration files are stored in the `~/.tendrils` folder

## `global-config.json` File
- `~/.tendrils/global-config.json`
- Contains default configuration values that are applied to actions in any [Tendrils repos](#tendrils-repo) unless otherwise specified
- This file is not usually version controlled as the configurations are mostly specific to the local computer

### `global-config.json` Schema
```json
{
    "default-repo-path": "path/to/default/repo"
}
```

#### `default-repo-path`
- Used to [specify](#specifying-the-tendrils-repo) the default tendrils repo if it is not otherwise provided
- Should be an absolute path, otherwise it will be [converted to one](#relative-paths)

# Developer Notes
- Prior to development, run the [`setup-tendrils.sh`](./dev/setup-tendrils.sh) script
- Running tests on Windows may require running in an elevated process due to Windows preventing the creation of symlinks without admin rights
    - Running the terminal as administrator will allow these tests to pass
    - Enabling developer mode will allow these tests to pass without running as administrator
        - Developer mode enables creating symlinks without admin rights
