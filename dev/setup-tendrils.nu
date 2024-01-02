cd $env.FILE_PWD
let os = $nu.os-info.name
if ($os == "windows") and ((is-admin) == false) {
    echo "Aborting: Requires running this script with admin priviledges"
    exit 1
}

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

if $os == "windows" {
    let user = $env.USERNAME
    ICACLS no_read_access.txt /inheritance:r
    ICACLS no_read_access_folder /inheritance:r
    ICACLS no_read_access.txt /grant $"($user):\(W)"
    ICACLS no_read_access_folder /grant $"($user):\(W)"
} else if $os == "macos" {
    chmod u-rw no_read_access.txt
    chmod u-rw no_read_access_folder
}
cd ..


echo "\tSetting up symlink samples"
mkdir SymlinksSource/original_folder
touch SymlinksSource/original_folder/misc.txt
touch SymlinksSource/original.txt
mkdir SymlinksDest/SomeApp/original_folder
touch SymlinksDest/SomeApp/original_folder/misc.txt
touch SymlinksDest/SomeApp/original.txt
if $os == "windows" {
    cd SymlinksSource
    mklink symfile.txt original.txt
    mklink /D symdir original_folder
    cd ../SymlinksDest/SomeApp
    mklink symfile.txt original.txt
    mklink /D symdir original_folder
    cd ../..
} else if $os == "macos" {
    cd SymlinksSource
    ln -s original.txt symfile.txt
    ln -s original_folder symdir
    cd ../SymlinksDest/SomeApp
    ln -s original.txt symfile.txt
    ln -s original_folder symdir
    cd ../..
}
