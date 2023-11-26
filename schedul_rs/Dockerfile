FROM rust:1.67

EXPOSE 6767
WORKDIR /app

COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo install --path .

COPY . .

RUN cargo install cargo-watch
CMD ["cargo", "watch", "-q", "-w", "src", "-x", "run" ]
