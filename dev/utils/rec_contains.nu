# Must be imported with a 'use rec_contains.nu' statement
# {first: John, last: Smith} | rec_contains "first" -> true
# {first: John, last: Smith} | rec_contains "abc" -> false

export def main [key: string] {
    try {
        $in | get $key | ignore
        true
    } catch {
        false
    }
}
