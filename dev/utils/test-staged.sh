# Tests the folder structure as staged in the git index
# This allows for properly testing partial commits

# HOW TO HANDLE IGNORED FILES? consider stash --all
# HOW TO HANDLE SUBMODULES?
# HOW TO HANDLE DEPENDENCIES?

repoFolder=$(git rev-parse --show-toplevel)
cd $repoFolder

testScriptPath="$repoFolder/dev/test-full.sh"

if [[ $(git diff) = "" ]]; then
    hasModifiedFiles=false
else
    hasModifiedFiles=true
fi

if [[ $(git ls-files --others --exclude-standard) = "" ]]; then
    hasUntrackedFiles=false
else
    hasUntrackedFiles=true
fi

if [[ $hasModifiedFiles = true || $hasUntrackedFiles = true ]]; then
    doStash=true
else
    doStash=false
fi

if [[ $doStash = true ]]; then
    echo "Partial commit detected - Stashing files"
    git stash --keep-index --include-untracked
else
    echo "Full commit detected - Nothing to stash"
fi

echo $"Running test script at: $testScriptPath"

echo ""
divider="------------------------------------------------------------------------"
echo "Test script output:"
echo $divider

sh $testScriptPath
testExitCode=$?

echo ""
echo $divider
if [[ $testExitCode = 0 ]]; then
    echo "Tests passed!"
else
    echo "Tests failed!"
    echo "Exit code: $testExitCode"
fi
echo $divider

if [[ $doStash = true ]]; then
    echo ""
    echo "Restoring from stash:"
    echo $divider
    git stash pop -q
fi
echo $divider

exit $testExitCode
