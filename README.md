# General
- *Tendrils* is a tool for synchronizing and version controlling small files/folder across multiple computers
- Main uses include:
    - Version controlling various configuration files
    - Version controlling small scripts that otherwise would not have their own repos
    - Quickly editing configuration files in a common place rather than tracking them down individually
- A *Tendrils* folder requires a [`tendrils.json`](#tendrilsjson) file

# Gathering
- Pulls all of the files specified in [`tendrils.json`](#tendrilsjson) to the [user data folder](#user-data-folder) where it can be compared to the Git index (using `git diff` or any other tool)
```bash
nu Tendrils.nu # Without arguments
```

# Spreading
- Copies all files specified in [`tendrils.json`](#tendrilsjson) from the [user data folder](#user-data-folder) to their respective folder on the computer
- This ***will*** overwrite any existing files
    - It is recommended to [gather](#gathering) before spreading to check that all settings are merged properly
- Using the [`--spread (-s)`](#spread--s) flag
```bash
nu Tendrils.nu -s
```

# `tendrils.json`
- Specifies all of the files and directories to be controlled
- Is stored in the [user data folder](#user-data-folder)
    - Must be at the top level of the folder

## Schema
```json
[
    {
        "app": "App Name",
        "name": "file or folder name",
        "parent-dirs-mac": ["/Users/<user>/path/to/item/parent/folder"],
        "parent-dirs-windows": ["C:\\Users\\<user>\\path 1",
                                "C:\\Users\\<user>\\path 2"],
        "folder-merge": false
    },
]
```

- `app`:
    - The name of the app that the item belongs to
    - Items in the [user data folder](#user-data-folder) will be grouped in subfolders based on this `app` name 
- `name`:
    - Must match the file or folder name
- `parent-dirs-<platform>`:
    - Must match the folder path containing the item in `name` (i.e. its parent folder)
    - The following variables will be resolved at runtime:
        - `<user>` - The name of the current user
        - Example: `/Users/<user>/Deskop`
    - If the list is empty, or if the path is blank, it will be skipped
    - During a [spread](#spreading) operation, the file/folder in the [user data folder](#user-data-folder) is copied to *all* paths in the list
    - During a [gather](#gathering) operation, only the *last* item in the list is considered
- `folder-merge`:
    - Specifies the merge strategy when folders are copied to or from the [user data folder](#user-data-folder)
    - This setting has no effect on the behaviour for included files - only folders
    - `true` - Add any new files, overwrite any conflicting files, but do not delete any files already in the destination folder
    - `false` - Entirely replace the destination folder with the source folder

## `tendrils-override.json`
- Items present in both `tendrils-override.json` and `tendrils.json` will respect the overriden values
    - Items present in `tendrils-override.json` but *not* in `tendrils.json` are ignored
- Is *not* version controlled
- Uses the same [schema](#schema) as [`tendrils.json`](#tendrilsjson)
- Is stored in the [user data folder](#user-data-folder)
    - Must be at the top level of the folder

# User Data Folder
- Stores all of the files/folders listed in the [`tendrils.json`](#tendrilsjson) file
- Items are grouped into subfolders by their [`app`](#schema) name
- It is recommended that the [user data folder](#user-data-folder) be under *Git* version control

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
- Used to [reset the user data folder](#resetting-the-folder)
    - Any changes in the application folders will be overwritten with the folder structure in [git](#version-control)
- Ignored if combined with the [`-d`](#dry--d) flag