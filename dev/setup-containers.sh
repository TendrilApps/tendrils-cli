set -e -u
repoFolder=$(git rev-parse --show-toplevel)
cd $repoFolder

# Setup dev container and build td CLI
docker build -f ./dev/Dockerfile.dev -t td-dev .
docker run -it --rm -v $repoFolder:/root/tendrils-cli td-dev cargo build

# Setup the example/playground container
docker build -f ./dev/Dockerfile.example -t td-example .

# Setup the GIF building container
docker build -f ./dev/Dockerfile.gifs -t td-gifs .
