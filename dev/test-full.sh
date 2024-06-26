exitCodeSum=0

cargo test --all-features -q --workspace
exitCodeSum=$(($exitCodeSum + $?))

# Check that the td binary builds (at least in debug mode)
cargo build --all-features 
exitCodeSum=$(($exitCodeSum + $?))

# Check that the tempdirs folder is empty
repoFolder=$(git rev-parse --show-toplevel)
if [[ "$(ls $repoFolder/target/tempdirs)" != "" ]]; then
    echo "Temp folder is not empty. Test cases may not be cleaning up properly."
    exitCodeSum=$(($exitCodeSum + 1))
fi

cargo doc --all-features --document-private-items --no-deps
exitCodeSum=$(($exitCodeSum + $?))

exit $exitCodeSum
