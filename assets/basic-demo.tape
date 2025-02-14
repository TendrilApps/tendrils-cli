# Use VHS to programmatically create a GIF of the terminal
# https://github.com/charmbracelet/vhs

Output assets/basic-demo.gif

# Set up a terminal with specific dimensions.
Set FontSize 14
Set Width 900
Set Height 850
Set BorderRadius 10

Type "ls ~/.config -a"  # Type a command in the terminal.
Sleep 500ms             # Pause for effect
Enter                   # Press "Enter"
Sleep 1500ms            # Display output

Type "td out"
Sleep 500ms
Enter
Sleep 2s

Type "ls ~/.config -a"
Sleep 500ms
Enter
Sleep 5s

# Type "td pull -l obsidian** -d"
# Type "td out -p -d"
# Type "td out -p alt-profile -d"
# Type "git s"
