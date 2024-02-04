# General
- *Tendrils* is a tool for synchronizing and version controlling small files/folders across multiple computers
- Main uses include:
    - Version controlling various configuration files
    - Version controlling small scripts that otherwise would not have their own repos
    - Quickly editing configuration files in a common place rather than tracking them down individually
- A *Tendrils* folder requires a [`tendrils.json`](#tendrilsjson) file

# Gathering
- Pulls all of the files specified in [`tendrils.json`](#tendrilsjson) to the [Tendrils folder](#tendrils-folder) where it can be compared to the Git index (using `git diff` or any other tool)
```bash
nu Tendrils.nu # Without arguments
```

# Spreading
- Copies all files specified in [`tendrils.json`](#tendrilsjson) from the [Tendrils folder](#tendrils-folder) to their respective folder on the computer
- This ***will*** overwrite any existing files
    - It is recommended to [gather](#gathering) before spreading to check that all settings are merged properly
- Using the [`--spread (-s)`](#spread--s) flag
```bash
nu Tendrils.nu -s
```

# `tendrils.json`
- Specifies all of the files and directories to be controlled
- Is stored in the [Tendrils folder](#tendrils-folder)
    - Must be at the top level of the folder

## Schema
```json
[
    {
        "group": "Group/app Name",
        "name": "file or folder name",
        "parent-dirs-mac": ["/Users/<user>/path/to/item/parent/folder"],
        "parent-dirs-windows": ["C:\\Users\\<user>\\path 1",
                                "C:\\Users\\<user>\\path 2"],
        "dir-merge": false
    },
]
```

- `group`:
    - The name of the group/app that the item belongs to
    - Items in the [Tendrils folder](#tendrils-folder) will be grouped in subfolders based on this `group` name 
- `name`:
    - Must match the file or folder name
- `parent-dirs-<platform>`:
    - Must match the folder path containing the item in `name` (i.e. its parent folder)
    - The following variables will be resolved at runtime:
        - `<user>` - The name of the current user
        - Example: `/Users/<user>/Deskop`
    - If the list is empty, it will be skipped
    - During a [spread](#spreading) operation, the file/folder in the [Tendrils folder](#tendrils-folder) is copied to *all* paths in the list
    - During a [gather](#gathering) operation, only the *first* item in the list is considered
- `dir-merge`:
    - Specifies the merge strategy when folders are copied to or from the [Tendrils folder](#tendrils-folder)
    - This setting has no effect on the behaviour for included files - only folders
    - `true` - Add any new files, overwrite any conflicting files, but do not delete any files already in the destination folder
    - `false` - Entirely replace the destination folder with the source folder

## `tendrils-override.json`
- Items present in both `tendrils-override.json` and `tendrils.json` will respect the overriden values
    - Items present in `tendrils-override.json` but *not* in `tendrils.json` are ignored
    - Items are considered "matched" if they share the same [`group`](#schema) and [`name`](#schema) fields
- Typically should not be version controlled
- Uses the same [schema](#schema) as [`tendrils.json`](#tendrilsjson)
- Is stored in the [Tendrils folder](#tendrils-folder)
    - Must be at the top level of the folder

# Tendrils Folder
- The folder containing the [`tendrils.json`](#tendrilsjson) file, the [`tendrils-override.json`](#tendrils-overridejson), and all of the files/folders controlled by their tendrils
- Items are grouped into subfolders by their [`group`](#schema) name

## Version Control
- The *Tendrils folder* can be placed under a version control system such as *Git*
    - In the case of *Git*, the `.git` folder would be at the top level of the *Tendrils folder*

## Resetting the Folder
- Using the [`--reset (-r)`](#reset--r) flag

```bash
nu Tendrils.nu -r
```

# Arguments
## `--compare (-c)`
- Used to print the output of the `git status` of the *Tendrils* repo after running all other operations 

## `--dry (-d)`
- Used to print the output of a [gather](#gathering) or [spread](#spreading) operation without actually moving any files

## `--spread (-s)`
- Used to perform a [spread](#spreading) operation

## `--reset (-r)`
- Used to [reset the Tendrils folder](#resetting-the-folder)
    - Any changes in the application folders will be overwritten with the folder structure in [git](#version-control)
- Ignored if combined with the [`-d`](#dry--d) flag

# Developer Notes
- Running tests on Windows may require running in an elevated process due to Windows preventing the creation of symlinks without admin rights
    - Running the terminal as administrator will allow these tests to pass
