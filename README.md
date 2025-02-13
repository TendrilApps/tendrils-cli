<div align="center" >
    <img src="./assets/logo.svg" width="200" />
</div>

# General
- Tendrils is a tool to flexibly manage specific files/folders strewn about the computer from a single location
    - Each file/folder is defined by a [tendril](#what-is-a-tendril) in the [`tendrils.json`](./docs/configuration.md#tendrilsjson) file
    - They are all stored in a common [Tendrils repo](#tendrils-repo)
    - `td` is the CLI tool to manage these tendrils
- Main uses include:
    - Versioning/syncing configuration files that are expected to be in specific locations on the machine (video game saves, application settings, `.bashrc`, `.vim`, etc)
    - Maintaining the same (or different) profiles across multiple machines
    - Maintaining different profiles for a single user on one machine
- Where Tendrils isn't as useful:
    - Ephemeral environments, such as Docker containers. It's likely simpler to use a one-off script/Dockerfile to put the files where they belong

# License & Acknowledgements
- GPLv3 - See additional licensing information in the [license file](./LICENSE.md)
- This project uses several open source libraries, and their licensing information can be found in the [third party license file](./LICENSE-3RD-PARTY.md)
- License and other info can be displayed using the `td about` command
``` bash
td about
```
- Example GIFs are made using [VHS](https://github.com/charmbracelet/vhs)

# What Is a Tendril?
- A tendril is simply a file or folder anywhere on the computer that is included in the scope of a [Tendrils repo](#tendrils-repo)

## Copy-Type Tendrils
- These tendrils rely on copying back and forth between the various locations on the computer and the [Tendrils repo](#tendrils-repo)
- Managed using the [`push`](./docs/tendrils-commands.md#pushing) and [`pull`](./docs/tendrils-commands.md#pulling) commands
- Designated by setting [`link`](./docs/configuration.md#link) to `false`

## Link-Type Tendrils
- These tendrils are setup as symlinks rather than being copied back and forth
- The symlinks are created at the various locations on the computer and all target the same file/folder in the [Tendrils repo](#tendrils-repo)
- Managed using the [`link`](./docs/tendrils-commands.md#linking) command
- Designated by setting [`link`](./docs/configuration.md#link) to `true`

# Tendrils Repo
- Serves as a common location for all of the tendrils defined in the [`tendrils.json`](./docs/configuration.md#tendrilsjson) file
- The master copies are stored here
- Any folder with a `.tendrils` subfolder containing a [`tendrils.json`](./docs/configuration.md#tendrilsjson) file is considered a Tendrils repo
    - Similar to how a Git repo has a `.git` folder at its top level
- The folder layout is up to the user - items are structured according to their [local path](./docs/configuration.md#local-path)
- You may have multiple repos on your system, depending on your needs
- See [specifying a tendrils repo](./docs/tendrils-commands.md#specifying-the-tendrils-repo)

## Version Control & Synchronization
- Tendrils itself does not have versioning or synchronization functionality across computers, but the Tendrils repo is often placed inside a synchronized folder such as OneDrive, under a version control system like Git, or on a USB stick

# Getting Started
## Supported Platforms
- Mac
- Windows
- Linux

## Installation
- The `td` CLI is a standalone binary
- Currently, this is only distributed through source code and must be built using the Cargo toolchain
- To build `td`:
``` bash
# From the 'tendrils' workspace folder
cargo build           # For a 'debug' build
cargo build --release # For a 'release' build
```

- By default, the output executable is placed in `target/debug` or `target/release` depending which profile was used to build it
- Once built, it is recommended to update your `PATH` variable to include this location (or copy the executable to somewhere in your `PATH`)
- Alternatively, just build and run the [example container](./dev/Dockerfile.example) to experiment before installing locally

## Set-up & Configuration
1. Create a new empty folder that will become the [Tendrils repo](#tendrils-repo)
2. `cd` into the new folder, then run the `td init` command
    -  This will create a starter configuration at [`.tendrils/tendrils.json`](./docs/configuration.md#tendrilsjson) inside the new folder
``` bash
cd MyTendrilsRepo
td init
```

3. Define some tendrils in the file following the [schema](./docs/configuration.md#tendrilsjson-schema)
    - See the [example configuration](./docs/example-repo/.tendrils/tendrils.json) for inspiration
4. Run a [`pull`](./docs/tendrils-commands.md#pulling) command to make an initial copy of any [copy-type](#copy-type-tendrils) tendrils to the [Tendrils repo](#tendrils-repo)
``` bash
td pull -d # Use the -d flag to dry-run at first
td pull
```
5. Run a [`link`](./docs/tendrils-commands.md#linking) command to setup any [link-type](#link-type-tendrils) tendrils
``` bash
td link -d # Use the -d flag to dry-run at first
td link
```
6. Optional: Create a [global configuration](./docs/configuration.md#global-configjson) to simplify some repeated commands

# Developers and Contribution
- See [developer notes](./docs/developers.md)

# More Use Cases
- Keeping Obsidian configurations synchronized across multiple vaults
- Versioning/syncing small scripts that otherwise would not have their own repos
- Editing miscellaneous files in a common place rather than tracking them down individually

# Future Plans
- Improved distribution
- `list` action to list all tendrils and their status
- `sync` action to automatically propage the last modified version of a file across all instances
- `diff` action to quickly compare all instances of a file
- More output formats
    - JSON
    - Plain text
- Get/set configuration using the CLI
- Respect XDG Base Directories
