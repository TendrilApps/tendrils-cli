# Must be imported with a 'use list_contains.nu' statement
# [a b c] | list_contains b -> true
# [a b c] | list_contains z -> false

export def main [value: string] {
    $in | any {|el| $el == $value}
}
