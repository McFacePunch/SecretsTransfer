FROM rust:latest as builder

WORKDIR /usr/src/app

# Install dependencies
COPY Cargo.toml Cargo.lock ./

# Build application
RUN mkdir src && \
    echo "fn main() {println!(\"if you see this, the build broke\")}" > src/main.rs && \
    cargo build --release && \
    rm -f target/release/deps/your_binary_name*
RUN cargo build --release

# Cleanup
RUN rm -f src/*.rs
COPY ./ .

# Build release container
RUN cargo build --release

FROM gcr.io/distroless/cc-debian11

COPY --from=builder /usr/src/app/target/release/your_binary_name /usr/local/bin/

# Service Port
EXPOSE 8443

CMD ["./SecureTransfer"]
