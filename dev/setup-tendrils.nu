cd $env.FILE_PWD


echo "Running generic repo setup"
nu ./generic/setup.nu


echo "Setting permissions for various sample files/folder"
cd ../tests/samples/NoReadAccess
touch no_read_access.txt
mkdir no_read_access_folder
touch ./no_read_access_folder/misc.txt

let os = $nu.os-info.name
if $os == "windows" {
    echo "Unimplemented on Windows"
} else if $os == "macos" {
    chmod u-rw no_read_access.txt
    chmod u-rw no_read_access_folder
}
