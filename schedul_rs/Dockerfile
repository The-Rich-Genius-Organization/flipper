FROM rust:1.67

WORKDIR /usr/src/app
COPY . .

RUN cargo install --path .

EXPOSE 6767
CMD ["cargo", "watch", "-q", "-c", "-x", "run", "-q"]
