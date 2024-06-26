# Use the latest version of the Rust base image
FROM rust:latest

# Set the working directory in the container to /my
WORKDIR /usr/src/ldk-node-hack-server

# Copy the Rust project files to the working directory
COPY src/ src/
COPY Cargo.toml .
COPY docker-config.json .

# Copy the in tree dependencies.
COPY protos/ protos/

# Build the Rust app
RUN cargo build

# Set the command to run the Rust app
CMD cargo run docker-config.json
