# Use VHS to programmatically create a GIF of the terminal
# https://github.com/charmbracelet/vhs

Output assets/profiles-demo.gif

# Set up a terminal with specific dimensions.
Set FontSize 14
Set Width 650
Set Height 600
Set BorderRadius 10

Type "td out -l vim**"
Sleep 500ms
Enter
Sleep 1s

Type "cat ~/.vimrc"
Sleep 500ms
Enter
Sleep 3s

Type "td out -l vim** -p alt-profile"
Sleep 500ms
Enter
Sleep 1s

Type "cat ~/.vimrc"
Sleep 500ms
Enter
Sleep 5s
