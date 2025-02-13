# General
- Configuration can occur at the repo level using the [`tendrils.json`](#tendrilsjson) file, or at the global level using the [`global-config.json`](#global-configjson) file
- Flexibility is a core focus to accomodate many different use cases across many systems

# `tendrils.json`
- Specifies all of the files and directories to be considered as tendrils
- Stored in the `.tendrils` folder inside a [Tendrils repo](../README.md#tendrils-repo)
    - `.tendrils/tendrils.json`
- See also [`global-config.json`](#global-configjson)

## `tendrils.json` Schema
- The json schema is intended to be flexible and to allow defining multiple tendrils in a compact form

```json
{
    "tendrils": {
        "SomeApp/SomeFile.ext": {
            "remotes": "/path/to/SomeFile.ext"
        },
        "SomeApp2/SomeFolder": {
            "remotes": [
                "/path/to/SomeFolder",
                "/path/to/DifferentName",
                "~/path/in/home/dir/SomeFolder",
                "/path/using/<MY-ENV-VAR>/SomeFolder"
            ],
            "dir-merge": false,
            "link": true,
            "profiles": ["home", "work"]
        },
        "SomeApp3/file.txt": [
            {
                "remotes": "~/unix/specific/path/file.txt",
                "link": true,
                "profiles": "unix"
            },
            {
                "remotes": "~/windows/specific/path/file.txt",
                "link": false,
                "profiles": "windows"
            }
        ]
    }
}
```
- Each entry in the `tendrils` dictionary above defines a set of tendrils
- The example above would define 7 tendrils
    - 1 tendril for `SomeApp/SomeFile.ext`
    - 4 tendrils for `SomeApp2/SomeFolder`
    - 2 tendrils for `SomeApp3/file.txt`
        - Note that unlike the first two, this entry is an array (using square brackets `[]` rather than `{}`) which allows specifying unique properties for the different remote locations
- `null` is not valid in any of these fields
- Must only contain valid UTF-8 characters
- See an example configuration file [here](./example-repo/.tendrils/tendrils.json)

### Local Path
- The keys in the `tendrils` dictionary represent the path to the master copy of the file/folder inside the [Tendrils Repo](../README.md#tendrils-repo)
    - This path is appended to the Tendrils repo
- If the path specifies multiple subfolders, the corresponding folder structure will be created
    - These paths should use `/` instead of `\` due to better cross-platform support
- Variables/other values in the local path are *not* [resolved](#path-resolving), but directory separators are [replaced](#directory-separators) on Windows
- The local path should not:
    - Be whitespace only
    - Point to the repo itself
    - Point to anything outside of the repo
    - Have path components containing only `..`
    - Point to the `.tendrils` folder or anything inside of it
- An attempt is made to detect values that break the guidelines above, but there are several edge cases that may not be detected
- See [Filtering by Local](./tendrils-commands.md#filtering-by-locals)
```json
"file.txt": {
    "remotes": "..."
}
```
Becomes `/path/to/tendrils/repo/file.txt`

``` json
"SomeFolder": {
    "remotes": "..."
}
```
Becomes `/path/to/tendrils/repo/SomeFolder`

``` json
"SomeFolder/file.txt": {
    "remotes": "..."
}
```
Becomes `/path/to/tendrils/repo/SomeFolder/file.txt`

### `remotes`
- A list of paths to the files/folders throughout the host machine that are associated with the corresponding [master copy](#local-path)
- Each remote defines a tendril
- Environment variables and some path abbreviations are supported
    - See [Path Resolving](#path-resolving)
- Remotes should be absolute paths
- Cross-platform paths should use `/` instead of `\` due to [the handling of directory separators](#directory-separators)
- Care must be taken to avoid [recursive tendrils](#recursive-tendrils)
- If the list is empty, no tendrils are defined
``` json
"remotes": ["~/some/specific/location/file.txt", "~/another/specific/location/file.txt"]
```
- If there is only one remote, the square brackets can be omitted:
``` json
"remotes": "~/some/specific/location/file.txt"
```

### `dir-merge`
- Specifies the merge strategy when folders are copied to or from the [Tendrils repo](../README.md#tendrils-repo)
- `true` - Add any new files, overwrite any conflicting files, but do not delete any files already in the destination folder
- `false` - Entirely replace the destination folder with the source folder
- If this field is omitted, it defaults to `false`
- This setting has no effect on the behaviour of file tendrils or link-type tendrils
    - It is only relevant for [copy-type](../README.md#copy-type-tendrils) folder tendrils
- Note: this field may be overriden depending on the value of [`link`](#link)

### `link`
- `true` - Designates these tendrils as [link-type](../README.md#link-type-tendrils)
    - This overrides any setting for [`dir-merge`](#dir-merge)
- `false` - Designates these tendrils as [copy-type](../README.md#copy-type-tendrils)
- If this field is omitted, it defaults to `false`

### `profiles`
- Provide an additional means for associating groups of tendrils
    - They may group by context, and often map to a specific computer (`home`, `work`, etc), or to a group of computers (`unix`, `windows`, etc)
    - They may also be used to group by category (`configs`, `scripts`, etc.)
- A tendril can belong to several profiles at once
- If no profiles are specified, these tendrils are always included regardless of which profiles are [specified](./tendrils-commands.md#filtering-by-profile) in the command
- If this field is omitted, it defaults to an empty list

``` json
"profiles": ["my-profile1", "my-profile2"]
```

- If there is only one profile, the square brackets can be omitted:
```json
"profiles": "my-profile"
```

# `global-config.json`
- Contains default configuration values that are applied to actions in any [Tendrils repos](../README.md#tendrils-repo) unless otherwise specified
- Stored in the `~/.tendrils` folder
    - `~/.tendrils/global-config.json`
- See also [`tendrils.json`](#tendrilsjson)

### `global-config.json` Schema
```json
{
    "default-repo-path": "path/to/default/repo",
    "default-profiles": ["common", "laptop"]
}
```

#### `default-repo-path`
- The default [tendrils repo path](./tendrils-commands.md#specifying-the-tendrils-repo) if it is not otherwise provided
- Allows calling `td` from anywhere
- Should be an absolute path, otherwise it will be [converted to one](#relative-paths)

#### `default-profiles`
- List of the default [profiles filter](./tendrils-commands.md#filtering-by-profile) if it is not otherwise provided
- Set this to the profiles specific to this host to prevent having to type them on every [command](./tendrils-commands.md)
- This is particularly useful if your tendril profiles are setup on a per-host basis like in [this example](./example-repo/.tendrils/tendrils.json)

# Path Resolving
- Paths will be resolved in the following order:
    1. Environment variables [are resolved](#resolving-environment-variables)
    2. A leading tilde (`~`) [is resolved](#resolving-tilde-)
    3. Relative paths are [converted to absolute](#relative-paths)
    4. Directory separators [are replaced](#directory-separators)
- These rules apply to [repo paths](./tendrils-commands.md#specifying-the-tendrils-repo) and [`remotes`](#remotes), but not to [local paths](#local-path)

## Resolving Environment Variables
- A [remote path](#remotes) or [repo path](./tendrils-commands.md#specifying-the-tendrils-repo) containing environment variables in the form `<VAR_NAME>` will be replaced with the variable values
``` json
"remotes": "<APPDATA>\\Local\\SomeApp\\file.txt"
```
- The above example will resolve to `C:\Users\YourUsername\AppData\Local\SomeApp\\file.txt` on a typical Windows installation
``` bash
td push --path <OneDrive>/MyRepo
```
- A path can contain multiple environment variables

## Resolving Tilde (`~`)
- A [remote path](#remotes) or a [repo path](./tendrils-commands.md#specifying-the-tendrils-repo) with a leading tilde will replace the `~` with the value of the `HOME` environment variable
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
- Recursive tendrils can cause [actions](./tendrils-commands.md#tendril-actions) to fail, or can cause unintuitive copying/linking behaviour
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
