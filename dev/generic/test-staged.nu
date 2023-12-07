# Tests the folder structure as staged in the git index
# This allows for properly testing partial commits
def main [
    --stash_only (-s), # Performs the stash operation but does not run tests
    testPath?: string # User specified path to a test file other than the default at <repoFolder>/dev/test-full.nu
                      # Relative links are resolved relative to the <repoFolder>
    ] {

    # HOW TO HANDLE IGNORED FILES? consider stash --all
    # HOW TO HANDLE SUBMODULES?
    # HOW TO HANDLE DEPENDENCIES?

    let repoFolder = (git rev-parse --show-toplevel)
    cd $repoFolder

    mut sep = "/"
    if ($nu.os-info.name == "windows") { $sep = "\\" }
    let-env testScriptPath = $testPath
    if ($env.testScriptPath == null) {
        $env.testScriptPath = $repoFolder + $sep + "dev" + $sep + "test-full.nu"
    }

    let hasModifiedFiles = ((git diff) != "")
    let hasUntrackedFiles = ((git ls-files --others --exclude-standard) != "")
    let doStash = ($hasModifiedFiles or $hasUntrackedFiles)

    if $doStash {
        "Partial commit detected - Stashing files"
        git stash --keep-index --include-untracked
    } else {
        "Full commit detected - Nothing to stash"
    }

    if ($stash_only) {
        "Working tree is in the state as would be committed"
        return
    }

    $"Running test script at: ($env.testScriptPath)"

    "\nTest script output:"
    let divider = "------------------------------------------------------------------------"
    $divider

    # Must execute as external command otherwise any exit statement
    # in the called script will exit all Nushells (possible bug?)
    # Unfortunately this prevents live printing of the test output
    # and will only deliver the results at the end
    let testResults = (do { nu $env.testScriptPath } | complete)
    $"(ansi dark_gray)($testResults.stdout)(ansi reset)"

    $divider
    if $testResults.exit_code == 0 {
        $"(ansi gb)Tests passed!(ansi reset)"
    } else {
        $"(ansi rb)Tests failed!(ansi reset)"
        $testResults.stderr
    }

    if $doStash {
        "Restoring from stash"
        git stash pop -q
    }

    $divider
    exit $testResults.exit_code
}