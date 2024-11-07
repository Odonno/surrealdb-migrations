FROM --platform=$BUILDPLATFORM rust:1.82 AS builder
ARG TARGETPLATFORM
ARG BUILDPLATFORM

RUN apt-get update && apt-get install -y musl-tools

RUN case "$TARGETPLATFORM" in \
    "linux/amd64")                   echo "x86_64-unknown-linux-musl" > /tmp/target ;; \
    "linux/arm64"|"linux/arm64/v8")  echo "aarch64-unknown-linux-musl" > /tmp/target ;; \
    *)                               echo "Unsupported platform: $TARGETPLATFORM" && exit 1 ;; \
    esac

RUN rustup target add $(cat /tmp/target)

WORKDIR /volume
ADD src src
ADD templates templates
ADD Cargo.toml Cargo.toml
ADD Cargo.lock Cargo.lock
RUN cargo build --release --target $(cat /tmp/target)

RUN mkdir -p /tmp/bin && cp /volume/target/$(cat /tmp/target)/release/surrealdb-migrations /tmp/bin/

FROM --platform=$TARGETPLATFORM alpine
COPY --from=builder /tmp/bin/surrealdb-migrations ./surrealdb-migrations
RUN chmod o+rx ./surrealdb-migrations
ENTRYPOINT [ "/surrealdb-migrations" ]