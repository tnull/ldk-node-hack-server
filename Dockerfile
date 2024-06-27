# Use the latest version of the Rust base image
FROM rust:latest

## Set the working directory in the container to /my
WORKDIR /usr/src/ldk-node-hack-server

# Copy the Rust project files to the working directory
COPY . .

# Build the Rust app
RUN cargo build --all
