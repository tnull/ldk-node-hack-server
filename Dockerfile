# Use the latest version of the Rust base image
FROM rust:latest

## Set the working directory in the container to /my
WORKDIR /usr/src/ldk-node-hack-server

# Copy the Rust project files to the working directory
COPY src/ src/
COPY Cargo.toml .
COPY docker-config.json .

# Copy the in tree dependencies.
COPY protos/ protos/

# Other workspace members. These are not dependencies but since 'cli' is included
# as member of the workspace, we need to copy it here for cargo build to work.
# TODO: Fix this, maybe by just removing 'cli' from the workspace.members
COPY cli/ cli/
COPY client/ client/ 

# Build the Rust app
RUN cargo build --all
