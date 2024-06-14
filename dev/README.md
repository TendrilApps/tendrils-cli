# 3rd-party-metadata.json
- The [3rd-party-metadata.json](./3rd-party-metadata.json) file is a combination of manually entered fields and auto-updated fields
- The [LICENSE-3RD-PARTY.md](../LICENSE-3RD-PARTY.md) file is generated from the metadata in the `.json` file
    - The markdown file should not be manually edited
- This allows quickly checking for any license changes when updating dependencies

## Schema
``` json
{
    "preamble": "Introductory sentence/paragraph",
    "cargo-dependencies": {
        "id-of-dependency-1": {
            "name": "Name of the dependency",
            "version": "x.y.z",
            "desc": "Short description of the package",
            "license": "MIT (for example)",
            "license_files": [
                "path/from/crate/root/LICENSE-MIT",
                "path/from/crate/root/LICENSE-APACHE",
                "https://www.web-link-to-license-raw-text.com"
            ],
            "repo": "https://link-to-source-code-repo.com"
        },
        "id-of-dependency-2": {
            "etc..."
        }
    }
}
```
- The `cargo-dependencies` section is automatically generated using the [3rd-party-update-cargo-deps.nu](./utils/3rd-party-update-cargo-deps.nu) script. In general this section should not be manually updated *except for* the `license_files` list
- The `license_files` can either be https URLs to the *raw* license text, or can be a file name that will be searched in the local repository relative to the crate root
    - For example, a value of `LICENSE.txt` would resolve typically resolve to `~/.cargo/registry/src/index.crates.io-6f17d22bba15001f/<crate-name>-<version>/LICENSE.txt`
        - This captures the exact license under which this specific version was distributed through crates.io
    - For https links, it is best to point this URL to the master branch (or equivalent), provided that the dependency version shares the same license as that in the master branch
        - This will help capture any future changes to the license file
        - Typically if using a recent version of a dependency, its license will match that in the master branch
- There must be at least one license file/URL specified
- If there are dual licenses that are "either-or", only include the `license_files` you plan to abide by (but do not change the `license` field - this is automatically populated and is mainly intended to capture changes to the licensing scheme in future dependency versions)
- This json metadata is then compiled to a markdown output using [3rd-party-compile-licenses.nu](./utils/3rd-party-compile-licenses.nu)

``` bash
nu 3rd-party-update-cargo-deps.nu path/to/3rd-party-metadata.json
nu 3rd-party-compile-licenses.nu path/to/3rd-party-metadata.json
```
