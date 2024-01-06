mut exitCodeSum = 0

cargo test --all-features
$exitCodeSum += $env.LAST_EXIT_CODE

exit $exitCodeSum
