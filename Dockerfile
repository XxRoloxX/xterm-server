FROM rust:buster as builder
WORKDIR /app
RUN apt-get update && apt-get install -y libssl-dev pkg-config
RUN rustup target add x86_64-unknown-linux-gnu

COPY . .
RUN cargo build --release --target x86_64-unknown-linux-gnu
RUN ls /app/target/release

FROM debian:buster as runtime
WORKDIR /app
RUN apt-get update && apt-get install -y openssl
COPY --from=builder /app /app
RUN echo 'export PS1="\[\033[01;32m\]\u@\h\[\033[00m\]:\[\033[01;34m\]\w\[\033[00m\]\$ "' >> /root/.bashrc
CMD ["/app/target/x86_64-unknown-linux-gnu/release/xterm-server"]

