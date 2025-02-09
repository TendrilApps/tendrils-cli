# Developer Notes
- Prior to development, run the [`setup-tendrils.sh`](../dev/setup-tendrils.sh) script
- Running tests on Windows may require running in an elevated process due to Windows preventing the creation of symlinks without admin rights
    - Running the terminal as administrator will allow these tests to pass
    - Enabling developer mode will allow these tests to pass without running as administrator
        - Developer mode enables creating symlinks without admin rights
- A [Dockerfile](../Dockerfile) is provided for testing on Linux
    - Certain tests have effects outside of the source code folder, so these will only run within this container to avoid cluttering the user's system
        - These must be run with the `_admin_tests` feature enabled
    - The rest of the test suite can be run on Linux normally (either inside or outside of a container)

# Contributing
- Not currently accepted, but will be in the future
