def main [third_party_mdata_path: string] {
    mut formatted_mdata = (open $third_party_mdata_path)

    let cargo_md = ((cargo metadata --all-features --format-version 1) | from json)
    let cargo_deps = (filter_workspace_crates $cargo_md)

    mut formatted_cargo_deps = {}
    for cargo_dep in $cargo_deps {
        # Preserve existing license file field for existing dependencies
        mut old_license_files = []
        if ($cargo_dep.id in $formatted_mdata.cargo-dependencies) {
            $old_license_files = ($formatted_mdata.cargo-dependencies | get $cargo_dep.id).license_files
        }

        $formatted_cargo_deps = ($formatted_cargo_deps | insert $cargo_dep.id {
            name: $cargo_dep.name
            version: $cargo_dep.version
            license: $cargo_dep.license
            license_files: $old_license_files # Preserve this field
            desc: $cargo_dep.description
            repo: $cargo_dep.repository
        })
    }

    $formatted_mdata.cargo-dependencies = (($formatted_mdata.cargo-dependencies | merge $formatted_cargo_deps) | sort)
    $formatted_mdata | save $third_party_mdata_path -f
}

# Removes the workspace crates and only leaves the third party crates
export def filter_workspace_crates [cargo_md: record] {
    let workspace_members = $cargo_md.workspace_members 
    let filtered = ($cargo_md.packages | where {|dep| $dep.id not-in $workspace_members })
    $filtered
}
