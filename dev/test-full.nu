mut exitCodeSum = 0

cargo test --all-features -q
$exitCodeSum += $env.LAST_EXIT_CODE

cargo doc --all-features --document-private-items --no-deps
$exitCodeSum += $env.LAST_EXIT_CODE

exit $exitCodeSum
