# Build shit
FROM rust:alpine AS builder

RUN apk add --no-cache musl-dev

WORKDIR /app

# Cache dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Build the binary
COPY src ./src
COPY static ./static
COPY Documents ./Documents
RUN touch src/main.rs && cargo build --release

# Run the project
FROM alpine:3.21

RUN addgroup -S yappl && adduser -S yappl -G yappl

WORKDIR /app

COPY --from=builder /app/target/release/third-year-project ./yappl

USER yappl

# Port/entrypoint stuff
EXPOSE 8080

ENTRYPOINT ["./yappl", "--web"]
CMD ["8080"]
