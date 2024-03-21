# General
- *Tendrils* provides a centralized location for working with, synchronizing, and version controlling files & folders dispersed across the computer
    - Each file/folder is defined by a tendril in the [`tendrils.json`](#tendrilsjson) file
    - They are all stored in a common [Tendrils folder](#tendrils-folder)
    - `td` is the CLI tool to manage these tendrils
- Main uses include:
    - Versioning/syncing configuration files that are expected to be in specific locations on the machine (`.bashrc`, `.vim`, etc)
    - Versioning/syncing small scripts that otherwise would not have their own repos
    - Quickly editing miscellaneous files in a common place rather than tracking them down individually

# Getting Started
## Installation
- Currently, *Tendrils* is only distributed through source code and must be built using the *Cargo* toolchain
- To build the `td` CLI:
``` bash
# From the 'tendrils' workspace folder
cargo build           # For a 'debug' build
cargo build --release # For a 'release' build
```
- By default, the output executable is placed in `target/debug` or `target/release` depending which profile was used to build it
- Once built, it is recommended to update your `PATH` variable to include this location (or copy the executable to somewhere in your `PATH`)

## Set Up
1. Create a new empty folder that will become the [Tendrils folder](#tendrils-folder)
2. `cd` into the new folder, then run the `td init` command
    -  This will create a starter [`tendrils.json`](#tendrilsjson) file inside the new folder
``` bash
cd MyTendrilsFolder
td init
```

3. Define some tendrils in the file following the [schema](#schema)
4. Run a [`pull`](#pulling) to make an initial copy of any [push/pull style](#pushpull-style) tendrils to the [Tendrils folder](#tendrils-folder)
5. Run a [`link`](#linking) to setup any [link style](#link-style) tendrils

# Tendrils Folder
- Serves as a common location for all of the tendrils defined in the [`tendrils.json`](#tendrilsjson) file
- Any folder with a [`tendrils.json`](#tendrilsjson) file at its top level is considered a Tendrils folder
- Items are grouped into subfolders by their [`group`](#schema) name

## Version Control & Synchronization
- *Tendrils* itself does not have versioning or synchronization functionality, but the *Tendrils* folder is often placed inside a synchronized folder such as *OneDrive*, or under a version control system such as *Git*
    - In the case of *Git*, the `.git` folder would be at the top level of the *Tendrils* folder

# Tendril Styles
## Push/Pull Style
- These tendrils rely on copying back and forth between the various locations on the computer and the [Tendrils folder](#tendrils-folder)
- Designated by setting [`link`](#link) to `false`

## Link Style
- These tendrils are setup as symlinks rather than being copied back and forth
- The symlinks are created at the various locations on the computer and all target the same file/folder in the [Tendrils folder](#tendrils-folder)
- Designated by setting [`link`](#link) to `true`

# `tendrils.json`
- Specifies all of the files and directories to be considered as tendrils
- Stored in the top level of the [Tendrils folder](#tendrils-folder)

## Schema
- The json schema is intended to be flexible and to allow defining multiple tendrils in a compact form
```json
[
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
] // Note the square outer brackets
```
- The example above would define 6 tendrils
    - One for each of the 3 names in each of the 2 parent folders
    - The profiles/tendril modes are applied to all of these tendrils

### `group`
- The name of the group/app that the tendril belongs to
- Items in the [Tendrils folder](#tendrils-folder) will be placed into subfolders with this `group` name 
- Group cannot be an empty string, cannot contain line breaks, and cannot be a path (i.e cannot contain `/` or `\`)
- Some specific values are invalid to prevent interfering with other common files/folders that may also be in the [Tendrils folder](#tendrils-folder). These invalid values are not case sensitive:
    - `tendrils.json`
    - `.git`

### `names`
- A list of file and/or folder names
- Each name defines a tendril at each [parent](#parents) location
- If the list is empty, no tendrils are defined
- If there is only one name, the square brackets can be omitted
``` json
"names": "file.txt"
```
- Names cannot be empty strings, cannot contain line breaks, and cannot be paths (i.e cannot contain `/` or `\`)

### `parents`
- A list of folder paths containing the files/subfolders in [`names`](#names) (i.e. their parent folder)
- Each parent defines a tendril for each name
- Environment variables and some path abbreviations are supported
    - See [Path Resolving](#path-resolving)
- If the list is empty, no tendrils are defined
- If there is only one parent, the square brackets can be omitted
``` json
"parents": "~/parent/folder"
```
- Parents cannot be empty strings, and cannot contain line breaks

### `dir-merge`
- Specifies the merge strategy when folders are copied to or from the [Tendrils folder](#tendrils-folder)
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
- Provide an additional means for associating groups of tendrils (in a broader sense than `group`
    - They may group by context, and often map to a specific computer (`home`, `work`, etc), or to a group of computers (`unix`, `windows`, etc)
    - They may also be used to group by category (`configs`, `scripts`, etc.)
- A tendril can belong to several profiles at once
- If no profiles are specified, these tendrils are always included regardless of which profiles are [specified](#filtering-by-profile) in the command
- If this field is omitted, it defaults to an empty list
- If there is only one profile, the square brackets can be omitted
```json
"profiles": "my-only-profile"
```

# Tendril Actions
- There are several actions for working with tendrils 
- `td` is the CLI tool that performs these commands
- Each action must be called from or pointed to a [Tendrils folder](#tendrils-folder)
    - See [specifying a tendrils folder](#specifying-the-tendrils-folder)

## Pulling
- Copies tendrils from their locations on the computer to the [Tendrils folder](#tendrils-folder)
- Only operates on [push/pull style](#pushpull-style) tendrils
- Only the *first* [parent](#parents) is used

```bash
td pull
```

## Pushing
- Copies tendrils from the *Tendrils* folder to their various locations on the machine
- Only operates on [push/pull style](#pushpull-style) tendrils
- *Each* [parent](#parents) is used
```bash
td push
```

## Linking
- Creates symlinks at the various locations on the computer to the tendrils in the [Tendrils folder](#tendrils-folder)
- Only operates on [link style](#link-style) tendrils
- *Each* [parent](#parents) is used
``` bash
td link
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

## Specifying the Tendrils Folder
- If no path argument is provided:
    1. *Tendrils* will first check if the current working directory is a [Tendrils folder](#tendrils-folder). If it is, this folder (and the tendrils defined in its [`tendrils.json`](#tendrilsjson)) will be used for the command
    2. If the CWD is not a *Tendrils* folder, then the `TENDRILS_FOLDER` environment variable will be checked
- A path can be explicitly set using the `--path` argument
    - Available on all of the actions listed above
    - If a path is provided, the current working directory and the value of the `TENDRILS_FOLDER` environment variable are not considered
``` bash
td push --path some/tendrils/folder
```

## Filtering Tendrils
### Filtering by Profile
- Using the `--profiles (-p)` argument
- Available on all of the actions listed above
- Only tendrils with one or more matching profiles will be included
- Tendrils without any profiles specified will still be included
``` bash
td push -p home mac
```
- The above example will include any tendrils with the `home` or `mac` profile, and any that don't have a profile

# Path Resolving
## Resolving Environment Variables
- A [parent](#parents) path containing environment variables in the form `<VAR_NAME>` will be replaced with the variable values
``` json
"parents": "<APPDATA>\\Local\\SomeApp"
```
- The above example will resolve to `C:\\Users\\YourUsername\\AppData\\Local\\SomeApp` on a typical *Windows* installation
- A path can contain multiple environment variables

## Resolving Tilde (`~`)
- A [parent](#parents) path with a leading tilde will replace the `~` with the value of the `HOME` environment variable
    - If `HOME` is not set, it will fall back to the combination of `HOMEDRIVE` and `HOMEPATH`
    - If either of those are not set, the raw path is used
``` json
"parents": "~/documents"
```
- The above example will resolve to `your/home/path/documents`

## Relative Paths
- Are not officially supported and their behaviour is undefined

# Developer Notes
- Running tests on Windows may require running in an elevated process due to Windows preventing the creation of symlinks without admin rights
    - Running the terminal as administrator will allow these tests to pass
    - Enabling developer mode will allow these tests to pass without running as administrator
        - Developer mode enables creating symlinks without admin rights
