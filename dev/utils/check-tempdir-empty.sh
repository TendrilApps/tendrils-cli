# Check that the tempdirs folder is empty
repoFolder=$(git rev-parse --show-toplevel)
if [ "$(ls $repoFolder/target/tempdirs)" != "" ]; then
    echo "Temp folder is not empty. Test cases may not be cleaning up properly."
    1/0
fi
