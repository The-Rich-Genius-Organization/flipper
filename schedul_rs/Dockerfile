# Building cargo base with cargo-watch
# FROM rust:1.67
# WORKDIR /app
# RUN cargo new --bin app
# RUN cargo install cargo-watch
# CMD ["tail", "-f", "/dev/null"]

FROM yohanderose/cargo-base:1.67

EXPOSE 6767
WORKDIR /app

COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo install --path .

COPY . .

CMD ["cargo", "watch", "-q", "-w", "src", "-x", "run" ]
