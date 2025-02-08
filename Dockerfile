FROM rust:latest

ENV CARGO_TARGET_DIR=target-container
ENV TENDRILS_TEST_CONTAINER=true
ENV PATH="$PATH:/root/tendrils-cli/target-container/debug"

WORKDIR /root/tendrils-cli

# Run these commands from the top level of the repo to setup container:
# docker build . -t td-dev
# docker run -it --rm -v .:/root/tendrils-cli td-dev bash
# cargo build/test/other (within the container)
