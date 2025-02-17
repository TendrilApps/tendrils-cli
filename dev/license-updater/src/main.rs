//! Updates 3rd party license file. Does not fully handle cases where there
//! are multiple versions of the same dependency - some manual work is
//! required to update the license files for these.

use clap::Parser;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

fn main() {
    // Get path to the 3rd party metadata file
    let args = LicenseUpdaterArgs::parse();

    if args.compile {
        update_3rd_party_licenses(get_metadata(), args.dry_run);
    }
    else {
        update_metadata_file(get_metadata(), args.dry_run);
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
struct FormattedDependency {
    /// Registry id of the crate, including the version info
    pub id: String,

    /// Name of the crate, without any version info
    pub name: String,

    /// SPDX license identifier (Apache-2.0, MIT, etc.).
    /// See <https://spdx.org/licenses/>
    pub license: Option<String>,

    /// List of license files, relative to the crate root in the registry
    #[serde(default)]
    pub license_files: Vec<String>,
    pub desc: Option<String>,
    pub src: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
struct CargoMetadataDependency {
    pub id: String,
    pub name: String,
    pub license: Option<String>,
    #[serde(default)]
    pub license_files: Vec<String>,
    #[serde(rename="description")]
    pub desc: Option<String>,
    #[serde(rename="repository")]
    pub repo: Option<String>,
    pub manifest_path: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
struct ThirdPartyMetadata {
    pub preamble: String,
    #[serde(rename="cargo-dependencies")]
    pub cargo_deps: Vec<FormattedDependency>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
struct CargoMetadataOutput {
    #[serde(rename="packages")]
    pub cargo_deps: Vec<CargoMetadataDependency>,

    /// List of crate id's that belong to the workspace
    pub workspace_members: Vec<String>,
}

/// Updates license metadata and Markdown files. Must be called from root of
/// repo.
#[derive(Parser, Clone, Debug, Eq, PartialEq)]
struct LicenseUpdaterArgs {
    /// Prints to stdout rather than writing to file
    #[arg(short, long)]
    dry_run: bool,

    /// Compiles license texts in Markdown format to file (will not update the metadata file)
    #[arg(short, long)]
    compile: bool,
}

fn fetch_https_license(url: &str) -> String {
    // Using curl isn't the "best" way to do this but it is simple.
    // Could be replaced in the future.
    let mut cmd = std::process::Command::new("curl");
    let output = cmd
        .arg(url)
        .arg("-s")
        .arg("-f")
        .output()
        .unwrap();
    if !output.status.success() {
        panic!("ERROR: Could not fetch license at {url}");
    }

    String::from_utf8(output.stdout).unwrap()
}

fn filter_workspace_members(
    deps: Vec<FormattedDependency>,
    workspace_members: Vec<String>,
) -> Vec<FormattedDependency> {
    deps.into_iter().filter(
        |dep| !workspace_members.contains(&dep.id)
    ).collect()
}

fn get_cargo_metadata() -> CargoMetadataOutput {
   let mut cmd = std::process::Command::new("cargo");
    let output = cmd
        .arg("metadata")
        .arg("--all-features")
        .arg("--format-version")
        .arg("1")
        .output()
        .unwrap();
    if !output.status.success() {
        panic!("ERROR: Could not get new metadata");
    }

    let json = String::from_utf8(output.stdout).unwrap();
    serde_json::from_str(&json).unwrap()
}

/// Returns license texts in the order they appear in the `license_files` field
fn get_license_texts(
    dep: &FormattedDependency,
    crate_path_lookup: &HashMap<String, PathBuf>,
) -> Vec<String> {
    if dep.license_files.is_empty() {
        panic!("ERROR: {} does not have any license files", dep.id)
    }

    let mut license_texts = Vec::with_capacity(dep.license_files.len());
    let crate_path = get_local_crate_path(dep, crate_path_lookup);
    for license_file in dep.license_files.iter() {
        let mut text;
        if is_https_url(&license_file) {
            text = fetch_https_license(&license_file);
        }
        else {
            let full_path = crate_path.join(license_file);
            text = std::fs::read_to_string(&full_path).expect(
                &format!("Could not find {}", full_path.to_string_lossy())
            );
        }
        text = String::from(text.trim_end());
        license_texts.push(text);
    }

    license_texts
}

/// Gets the path to the crate in the local registry
fn get_local_crate_path(
    dep: &FormattedDependency,
    crate_path_lookup: &HashMap<String, PathBuf>,
) -> PathBuf {
    crate_path_lookup.get(&dep.id).unwrap().parent().unwrap().into()
}

fn get_metadata() -> ThirdPartyMetadata {
    let mdata_path = get_metadata_path();
    let json = std::fs::read_to_string(mdata_path).unwrap();
    let de = serde_json::from_str::<ThirdPartyMetadata>(&json).unwrap();
    de
}

fn get_metadata_path() -> PathBuf {
    std::env::current_dir().unwrap().join("dev/3rd-party-metadata.json")
}

fn get_compiled_licenses_path() -> PathBuf {
    std::env::current_dir().unwrap().join("LICENSE-3RD-PARTY.md")
}

fn get_dep_lookup(
    deps: Vec<FormattedDependency>
) -> HashMap<String, FormattedDependency> {
    let mut lookup = HashMap::with_capacity(deps.len());

    for dep in deps {
        lookup.insert(dep.name.clone(), dep);
    }

    lookup
}

fn get_local_crate_path_lookup(
    deps: Vec<CargoMetadataDependency>
) -> HashMap<String, PathBuf> {
    let mut lookup = HashMap::with_capacity(deps.len());
    for dep in deps {
        lookup.insert(dep.id, PathBuf::from(dep.manifest_path));
    }

    lookup
}

/// Crude check for https urls
fn is_https_url(path: &str) -> bool {
    path.starts_with("https:")
}

/// Merge old license files (if applicable) onto the new versions of the
/// dependencies. Check if there is a link to the repo, or default to
/// docs.rs for the source code.
fn merge_info(
    deps: Vec<CargoMetadataDependency>,
    name_lookup: HashMap<String, FormattedDependency>,
) -> Vec<FormattedDependency> {
    let mut formatted_deps = Vec::with_capacity(deps.len());

    for dep in deps {
        // Check if old license matches new license to determine if same
        // license file paths can be used
        let old_dep = name_lookup.get(&dep.name);
        let license_files = match old_dep {
            Some(v) => {
                if v.license == dep.license {
                    v.license_files.clone()
                }
                else {
                    vec![]
                }
            }
            _ => vec![],
        };

        // Default to docs.rs if there is not a provided repo:
        let src = match dep.repo {
            Some(v) => v,
            _ => format!("https://docs.rs/{}", dep.name),
        };

        let formatted_dep = FormattedDependency {
            id: dep.id,
            name: dep.name.clone(),
            license: dep.license,
            license_files,
            desc: dep.desc,
            src,
        };

        formatted_deps.push(formatted_dep);
    }

    formatted_deps
}

/// Formats a dependency and its license as a Markdown section.
fn to_markdown(dep: &FormattedDependency, license_texts: Vec<String>) -> String {
    let mut output = format!("## {}\n", dep.name);
    output.push_str(&format!("The `{}` software is included in this product.\n", dep.name));
    output.push_str(&format!("The source code is available here: {}.\n", dep.src));
    output.push_str("Its license(s) and notice(s) are as follows:\n\n");

    for (i, text) in license_texts.iter().enumerate() {
        output.push_str(&text);
        output.push_str("\n\n\n");

        // Add divider between multiple licenses on the same dependency
        if i < license_texts.len() - 1 {
            output.push_str("---\n");
        }
    }

    output
}

/// Updates the metadata file and returns the updated metadata.
fn update_metadata_file(old_metadata: ThirdPartyMetadata, dry_run: bool) -> ThirdPartyMetadata {
    let old_deps = old_metadata.cargo_deps;
    let name_lookup = get_dep_lookup(old_deps);

    let cargo_metadata = get_cargo_metadata();
    let mut new_deps = merge_info(
        cargo_metadata.cargo_deps,
        name_lookup,
    );
    new_deps = filter_workspace_members(new_deps, cargo_metadata.workspace_members);
    new_deps.sort_by(|a, b| a.id.cmp(&b.id));
    let new_metadata = ThirdPartyMetadata {
        cargo_deps: new_deps,
        ..old_metadata
    };

    let json = serde_json::to_string_pretty(&new_metadata).unwrap();

    if dry_run {
        println!("{}", json);
    }
    else {
        std::fs::write(get_metadata_path(), json).unwrap();
    }
    new_metadata
}

fn update_3rd_party_licenses(metadata: ThirdPartyMetadata, dry_run: bool) {
    let mut output = String::from("<!-- This file is auto-generated using 3rd-party-compile-licenses.nu - any changes here will be overwritten. -->\n\n");
    output.push_str("# General\n");
    output.push_str(&metadata.preamble);
    output.push_str("\n\n# Third Party Dependencies\n");

    let crate_path_lookup = get_local_crate_path_lookup(
        get_cargo_metadata().cargo_deps
    );

    for dep in metadata.cargo_deps {
        let dep_license_texts = get_license_texts(&dep, &crate_path_lookup);
        let md_text = to_markdown(&dep, dep_license_texts);
        output.push_str(&md_text);
    }

    if dry_run {
        println!("{}", output);
    }
    else {
        std::fs::write(get_compiled_licenses_path(), output).unwrap();
    }
}
