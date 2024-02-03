cd $env.FILE_PWD
let os = $nu.os-info.name

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
