cd $env.FILE_PWD


echo "Running generic repo setup"
nu ./generic/setup.nu


echo "Setting up sample files/folders:"
cd ../tests/samples

echo "\tSetting up NoReadAccess samples"
mkdir NoReadAccess
cd NoReadAccess
touch no_read_access.txt
mkdir no_read_access_folder
touch no_read_access_folder/misc.txt

# Requires that Git Bash or equivalent is installed
# on Windows (and is in PATH) in order to get the chmod executable
chmod u-rw no_read_access.txt
chmod u-rw no_read_access_folder
cd ..

echo "\tSetting up symlink samples"
mkdir SymlinksSource/original_folder
touch SymlinksSource/original_folder/misc.txt
touch SymlinksSource/original.txt
mkdir SymlinksDest/SomeApp/original_folder
touch SymlinksDest/SomeApp/original_folder/misc.txt
touch SymlinksDest/SomeApp/original.txt
let os = $nu.os-info.name
if $os == "windows" {
    if (is-admin) == false {
        echo "\tAborting: Requires running this script with admin priviledges"
        exit 1
    }
    cd SymlinksSource
    mklink symfile.txt original.txt
    mklink /D symdir original_folder
    cd ../SymlinksDest/SomeApp
    mklink symfile.txt original.txt
    mklink /D symdir original_folder
    cd ../..
} else if $os == "macos" {
    echo "Unimplemented on Mac"
}
