FROM rust:1.65 as builder

WORKDIR /home/tlint/

COPY . .
RUN cargo build --release

FROM registry.suse.com/bci/rust:latest

WORKDIR /home/tlint/
COPY --from=builder /home/tlint/target/release/tlint .
WORKDIR /data
VOLUME ["/data"]
ENTRYPOINT ["/home/tlint/tlint"]
