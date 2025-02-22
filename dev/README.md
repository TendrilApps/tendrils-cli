# Developer Notes
- Running tests on Windows may require running in an elevated process due to Windows preventing the creation of symlinks without admin rights
    - Running the terminal as administrator will allow these tests to pass
    - Enabling developer mode will allow these tests to pass without running as administrator
        - Developer mode enables creating symlinks without admin rights
- A [Dockerfile](../dev/Dockerfile.dev) is provided for testing in a Linux environment
    - Certain tests have effects outside of the source code folder, so these will only run within this container to avoid cluttering the user's system
        - These must be run with the `_admin_tests` feature enabled
    - The rest of the test suite can be run on Linux normally (either inside or outside of a container)
- The [`test-full.sh`](./test-full.sh) script will emulate the CI commands that will be run in Github actions
    - This script can be used to test locally first

# Contributing
- Not currently accepted, but will be in the future

# 3rd-party-metadata.json
- The [3rd-party-metadata.json](./3rd-party-metadata.json) file is a combination of manually entered fields and auto-updated fields
- The [LICENSE-3RD-PARTY.md](../LICENSE-3RD-PARTY.md) file is generated from the metadata in the `.json` file
    - The markdown file should not be manually edited
- This allows quickly checking for any license changes when updating dependencies

## Schema
``` json
{
    "preamble": "Introductory sentence/paragraph",
    "cargo-dependencies": [
        {
            "id": "registry+url#dep1@version",
            "name": "Name of the dependency",
            "desc": "Short description of the package",
            "license": "MIT (for example)",
            "license_files": [
                "path/from/crate/root/LICENSE-MIT",
                "path/from/crate/root/LICENSE-APACHE",
                "https://www.web-link-to-license-raw-text.com"
            ],
            "src": "https://link-to-source-code-repo.com"
        },
        {
            "id": "registry+url#dep2@version",
            "etc..."
        }
    ]
}
```
- The `cargo-dependencies` section is automatically generated using the `license-updater` local crate. In general this section should not be manually updated *except for* the `license_files` list
- The `license_files` can either be https URLs to the *raw* license text, or can be a file name that will be searched in the local repository relative to the crate root
    - For example, a value of `LICENSE.txt` would resolve typically resolve to `~/.cargo/registry/src/index.crates.io-6f17d22bba15001f/<crate-name>-<version>/LICENSE.txt`
        - This captures the exact license under which this specific version was distributed through crates.io
    - For https links, it is best to point this URL to the master branch (or equivalent), provided that the dependency version shares the same license as that in the master branch
        - This will help capture any future changes to the license file
        - Typically if using a recent version of a dependency, its license will match that in the master branch
- There must be at least one license file/URL specified
- If there are dual licenses that are "either-or", only include the `license_files` you plan to abide by (but do not change the `license` field - this is automatically populated and is mainly intended to capture changes to the licensing scheme in future dependency versions)
- This json metadata is then compiled to a markdown output using the `license-updater` local crate

``` bash
# From the root of the repo
cargo run -p license-updater -- -d      # Dry-run - confirm metadata output
cargo run -p license-updater            # Update metadata file
cargo run -p license-updater -- -c -d   # Dry-run - confirm Markdown output
cargo run -p license-updater -- -c      # Update third party license file
```

# Example GIFs
- The example GIFs used in the docs can be updated using [this container](./Dockerfile.gifs)
