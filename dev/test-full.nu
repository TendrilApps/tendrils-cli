mut exitCodeSum = 0

cargo test
$exitCodeSum += $env.LAST_EXIT_CODE

exit $exitCodeSum
