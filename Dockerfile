FROM clux/muslrust:1.81.0-stable as builder
WORKDIR /volume
ADD src src
ADD templates templates
ADD Cargo.toml Cargo.toml
ADD Cargo.lock Cargo.lock
RUN cargo build --release --target x86_64-unknown-linux-musl

FROM alpine
COPY --from=builder /volume/target/x86_64-unknown-linux-musl/release/surrealdb-migrations ./surrealdb-migrations
RUN chmod o+rx ./surrealdb-migrations
ENTRYPOINT [ "/surrealdb-migrations" ]