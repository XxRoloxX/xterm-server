FROM rust:buster as builder
WORKDIR /app
RUN apt-get update && apt-get install -y libssl-dev pkg-config
RUN rustup target add x86_64-unknown-linux-gnu
COPY . .
RUN cargo build --release --target x86_64-unknown-linux-gnu
RUN ls /app/target/release

FROM debian:buster as runtime
WORKDIR /app
# The binary is not 100% static, so we need to install openssl
RUN apt-get update && apt-get install -y openssl htop
COPY --from=builder /app /app
# Set the port for xterm server (or ovveride with -e XTERM_PORT=...)
# ENV XTERM_PORT=3666
ENV TERM=xterm-256color
# Add custom prompt
RUN echo 'export PS1="\[\033[01;32m\]\u@\h\[\033[00m\]:\[\033[01;34m\]\w\[\033[00m\]\$ "' >> /root/.bashrc
CMD ["/app/target/x86_64-unknown-linux-gnu/release/xterm-server"]

