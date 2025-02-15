set -e -u
repoFolder=$(git rev-parse --show-toplevel)
cd $repoFolder

cargo test --all-features -q --workspace --release

# Check that the td binary builds
cargo build --all-features --release

# Check that the tempdirs folder is empty
if [ "$(ls $repoFolder/target/tempdirs)" != "" ]; then
    echo "Temp folder is not empty. Test cases may not be cleaning up properly."
    1/0
fi

cargo doc --all-features --document-private-items --no-deps --workspace --release
