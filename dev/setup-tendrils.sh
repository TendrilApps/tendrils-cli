cd "$(dirname $0)"
os="$(sh ./Generic/get-os.sh)"

echo "Running generic repo setup"
sh ./generic/setup.sh


echo "Setting up sample files/folders:"
cd ../tests/samples

echo "    Setting up NoReadAccess samples"
mkdir NoReadAccess
cd NoReadAccess
touch no_read_access.txt
mkdir no_read_access_dir
touch no_read_access_dir/misc.txt

if [[ $os == "windows" ]]; then
    user=$(logname)
    echo $user
    ICACLS no_read_access.txt //inheritance:r
    ICACLS no_read_access_dir //inheritance:r
    ICACLS no_read_access.txt //grant "$user:(W)"
    ICACLS no_read_access_dir //grant "$user:(W)"
elif [[ $os == "osx" ]]; then
    chmod u-rw no_read_access.txt
    chmod u-rw no_read_access_dir
fi
