set -e -u

# These steps should mirror ../.github/workflows/ci.yml
sh dev/utils/test-except-admin.sh
sh dev/utils/build-td.sh
sh dev/utils/build-docs.sh
sh dev/utils/check-tempdir-empty.sh
