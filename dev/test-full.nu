mut exitCodeSum = 0

cargo test --all-features -q
$exitCodeSum += $env.LAST_EXIT_CODE

# Check that the tempdirs folder is empty
if (((ls ./target/tempdirs) | length) != 0) {
    "Temp folder is not empty. Test cases may not be cleaning up properly."
    $exitCodeSum += 1
}

cargo doc --all-features --document-private-items --no-deps
$exitCodeSum += $env.LAST_EXIT_CODE

exit $exitCodeSum
