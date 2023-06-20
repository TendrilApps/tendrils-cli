def main [
    --spread (-s) # Spreads the files in the UserData folder to their respective folders on the computer (otherwise it gathers the files)
    --compare (-c) # Runs a 'git status' report for the Tendrils repo after other operations are complete
    --dry (-d) # Dry-run - Outputs what would be done, but does not transfer any files
    --reset (-r)    # Reset the UserData folder to match that in Git.
                    # If -s is ommitted, the folder is reset and the gather is skipped.
                    # Ignored if combined with -d
    ] {

    let-env dirSep = ""
    let-env userPath = ""

    # Platform specific setup
    if (getOs) == "windows" {
        $env.dirSep = "\\"
    } else if (getOs) == "macos" {
        $env.dirSep = "/"
    }

    let-env tendrilsFolder = $env.FILE_PWD
    let-env userDataFolder = $env.PWD
    let-env includeFile = ($env.userDataFolder + $env.dirSep + "tendrils.json")
    let-env includeFileLocal = ($env.userDataFolder + $env.dirSep + "tendrils-override.json")

    if (not $dry) and $reset {
        resetUserDataFolder

        if not $spread { return }
    }

    let results = (copyItems (importIncludedItems) $spread $dry)

    $results | each {|result|
        report $result
    }

    if $compare {
        reportGitStatus
    }
}

def copyItems [includedItems: list, spread: bool, dryRun: bool] {
    mut results = []

    mut i = 0
    loop {
        if $i >= ($includedItems | length) {
            break
        }

        let includedItem = ($includedItems | get $i)
        mut localItemPaths = $includedItem.localItemPaths

        if ($includedItem.localItemPaths | is-empty) {
            # Display but skip the included item
            $localItemPaths = [""]
        } else if not $spread {
            # Only gather from the last local path given
            $localItemPaths = ($localItemPaths | last 1)
        }

        mut j = 0
        loop {
            if $j >= ($localItemPaths | length) {
                break
            }

            let localItemPath = ($localItemPaths| get $j)
            mut sourcePath = $localItemPath
            mut destPath = $includedItem.controlledItemPath

            if $spread {
                $sourcePath = $includedItem.controlledItemPath
                $destPath = $localItemPath
            }

            mut result = {
                includedItem: $includedItem
                from: $sourcePath,
                to: $destPath,
                success: "",
                spread: $spread
            }

            if (($localItemPath == "") or $dryRun) {
                $result.success = "skipped"
            } else {
                if (($includedItem.pathType == "dir") and (not $includedItem.folderMerge)) {
                    rm ($destPath) -r -f
                }

                if ((not ($includedItem.controlledItemPath | path exists)) and (not $spread)) {
                    mkdir ($includedItem.controlledItemPath | path dirname)
                }

                $result.success = ((copyItem $sourcePath $destPath) | into string)
            }

            $results = ($results | append $result)

            $j += 1
        }

        $i += 1
    }

    return $results
}

def copyItem [from: string, to: string] {
    let pathType = ($from | path type)
    mut success = false

    if $pathType == "file" {
        cp $from $to
        $success = true
    } else if $pathType == "dir" {
        # Remove the folder name from the path so that
        # the folder is copied to the right level
        cp $from ($to | path dirname) -r
        $success = true
    } else {
        # echo $"($to) either does not exist, or is a symlink"
    }

    return $success
}

def getLocalOverride [includeItem: record, includeOverrides: list] {
    mut i = 0
    loop {
        if $i >= ($includeOverrides | length) {
            break
        }

        let overrideItem = ($includeOverrides | get $i)
        if (($includeItem.name == $overrideItem.name) and ($includeItem.app == $overrideItem.app)) {
            return $overrideItem
        }

        $i += 1
    }

    return null
}

def importIncludedItems [] {
    mut includedItems = []

    if ($env.includeFile | path exists) {
    } else {
        echo $"($env.includeFile) was not found."
        exit
    }

    let rawImports = (open $env.includeFile).items
    mut rawOverrideImports = []

    if ($env.includeFileLocal | path exists) {
        $rawOverrideImports = (open $env.includeFileLocal).items
    }

    mut i = 0
    loop {
        if $i >= ($rawImports | length) {
            break
        }

        let rawItem = ($rawImports | get $i)
        let rawOverrideItem = getLocalOverride $rawItem $rawOverrideImports

        if $rawOverrideItem == null {
            $includedItems = ($includedItems | append (parseImportedItem $rawItem))
        } else {
            $includedItems = ($includedItems | append (parseImportedItem $rawOverrideItem))
        }

        $i += 1
    }

    return $includedItems
}

def getLocalItemPaths [includeItem: record] {
    mut rawPaths = []
    mut username = ""

    if (getOs) == "windows" {
        $rawPaths = $includeItem.parent-dirs-windows
        $username = $env.USERNAME
    } else if (getOs) == "macos" {
        $rawPaths = $includeItem.parent-dirs-mac
        $username = $env.USER
    }

    mut i = 0
    loop {
        if $i >= ($rawPaths | length) {
            break
        }

        # Format the raw paths
        $rawPaths = ($rawPaths | update $i (($rawPaths | get $i) + $env.dirSep + $includeItem.name))
        $rawPaths = ($rawPaths | update $i (($rawPaths | get $i) | str replace "<user>" $username))

        $i += 1
    }

    return $rawPaths
}

def getOs [] {
    return $nu.os-info.name
}

def getPathRelToScript [path: string] {
    # return ("." + $env.dirSep + ($path | path relative-to $env.FILE_PWD))
    return $path
}

# Converts the items imported from the schema to
# a structure used within the script
def parseImportedItem [rawItem: record] {
    let localItemPaths = getLocalItemPaths $rawItem
    let controlledItemPath = $env.userDataFolder + $env.dirSep + $rawItem.app + $env.dirSep + $rawItem.name

    mut pathType = ($controlledItemPath | path type)
    if (($pathType == "") and (not ($localItemPaths | is-empty))) {
        $pathType = (($localItemPaths | last 1).0 | path type)
    }

    if (($rawItem.app | str upcase) == ".GIT") {
        echo $"An invalid entry was found. ($rawItem.app) is not a valid app name."
        exit
    }

    return {
        app: $rawItem.app,
        name: $rawItem.name,
        controlledItemPath: $controlledItemPath,
        localItemPaths: $localItemPaths,
        pathType: $pathType,
        folderMerge: $rawItem.folder-merge
    }
}

def report [result: record] {
    mut dispResult = {
        App: $result.includedItem.app,
        Name: $result.includedItem.name,
        From: "",
        To: "",
        Type: $result.includedItem.pathType,
        Success: ""
    }

    # Display controlled items as relative links
    if $result.spread {
        $dispResult.From =  (getPathRelToScript $result.from)
        $dispResult.To = $result.to
    } else {
        $dispResult.From = $result.from
        $dispResult.To = (getPathRelToScript $result.to)
    }

    # Colour code success values
    if ($result.success | into string) == "true" {
        $dispResult.Success = "true"
    } else if $result.success == "skipped" {
        $dispResult.Success = $"(ansi yellow)skipped(ansi reset)"
    } else {
        $dispResult.Success = $"(ansi red)($result.success)(ansi reset)"
    }

    echo $dispResult
}

def reportGitStatus [] {
    cd $env.userDataFolder
    git status
}

def resetUserDataFolder [] {
    echo $"Resetting ($env.userDataFolder)"
    # TODO: Change so that it only deletes the folders by *app*
    echo "Resetting is not currently supported"
    # rm $env.userDataFolder -r -f
    # git checkout $env.userDataFolder
}
