FROM rust:latest as builder

WORKDIR /usr/src/app

# Copy over the `Cargo.toml` and `Cargo.lock` files to fetch and build dependencies
COPY Cargo.toml Cargo.lock ./

# Build application
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release

# Cleanup
RUN rm -f src/*.rs
COPY ./ .

# Build final container
RUN cargo build --release

# Start a new build stage, using Google's distroless base image
FROM gcr.io/distroless/cc-debian11

# Copy the binary from the builder stage to the current stage
COPY --from=builder /usr/src/app/target/release/your_binary_name /usr/local/bin/

# Service Port
EXPOSE 8443

# Define the command to run on container start
CMD ["./SecureTransfer"]
