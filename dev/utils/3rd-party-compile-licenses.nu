use 3rd-party-update-cargo-deps.nu filter_workspace_crates

def main [formatted_mdata_path: string] {
    let formatted_mdata = (open $formatted_mdata_path)
    mut formatted_cargo_deps = $formatted_mdata.cargo-dependencies

    let cargo_md = ((cargo metadata --all-features --format-version 1) | from json)
    let cargo_deps = (filter_workspace_crates $cargo_md)

    # Attach the cargo dependency to the formatted dependency
    for cargo_dep in $cargo_deps {
        mut formatted_match = ($formatted_cargo_deps | get $cargo_dep.id)
        $formatted_match = ($formatted_match | insert "cargo_dep" $cargo_dep)
        $formatted_cargo_deps = ($formatted_cargo_deps | update $cargo_dep.id $formatted_match)
    }

    let formatted_cargo_deps_list = ($formatted_cargo_deps | transpose id dep)

    echo $"<!-- This file is auto-generated using 3rd-party-compile-licenses.nu - any changes here will be overwritten. -->\n"
    echo "# General"
    echo $formatted_mdata.preamble

    echo "\n# Third Party Dependencies"
    for id_dep_pair in $formatted_cargo_deps_list {
        # Attach license text to the dependency and print it
        mut dep = $id_dep_pair.dep
        let lic_txt = (get_license_texts $dep)
        $dep = ($dep | insert license_txt $lic_txt)
        $dep | to_md
    }
}

# Supports a crate that has the license nested up to 1 deep
# (i.e. crate_folder/src/LICENSE, up to crate_folder/LICENSE)
def get_license_texts [dep: record] {
    let local_src_path = (get_local_src_path $dep.cargo_dep)

    if (($dep.license_files | length) < 1) {
        $"Error: ($dep.name) does not have any license files"
        1 / 0
    }

    mut lic_txt = ""

    # Check if it's an https URL first, otherwise check local crates
    for lic_file in $dep.license_files {
        let lic_path1 = ($local_src_path | path dirname | path join $lic_file)
        let lic_path2 = ($local_src_path | path dirname | path dirname | path join $lic_file)

        if ($lic_file | is_https_url) {
            $lic_txt = ($lic_txt + (curl $lic_file -s -f))
            if ($env.LAST_EXIT_CODE != 0) {
                $"Could not fetch $($lic_file) from the given URL"
                1 / 0
            }
        } else if ($lic_path1 | path exists) {
            $lic_txt = ($lic_txt + (open $lic_path1))
        } else {
            # Last resort - fail if this doesn't exist
            try {
                $lic_txt = ($lic_txt + (open $lic_path2))
            } catch {
                echo $"\n\nCould not open ($lic_path2)"
                1 / 0
            }
        }
    }

    $lic_txt
}

def get_local_src_path [cargo_dep: record] {
    let depended_targets = ($cargo_dep.targets | where {|t| (($t.kind.0 == "lib") or ($t.kind.0 == "proc-macro")) })
    if ($depended_targets | is-empty) {
        echo $"Error: The depended target for ($cargo_dep.name) could not be determined as there were multiple viable targets"
        exit 1
    } else if ($depended_targets | length) > 1 {
        echo $"Error: No viable targets were found for ($cargo_dep.name)"
        exit 1
    }

    $depended_targets.0.src_path
}

# Accepts a piped dependency and formats the info
# and license as a markdown section
def to_md [] {
    let dep = $in
    echo $"## ($dep.name) - v($dep.version)"
    echo $"The `($dep.name)` software is included in this product."
    echo $"The source code is available here: ($dep.repo)."
    echo "Its license(s) and notice(s) are as follows:\n"
    $dep.license_txt
    echo ""
}

# Accepts a piped string and returns true if it is an http url
def is_https_url [] {
    try {
        let url_info = ($in | url parse)
        $url_info.scheme == 'https'
    } catch {
        false
    }
}
