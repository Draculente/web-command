FROM rust:1.74

ENV PATH=$PATH:~/.cargo/bin

WORKDIR /app

COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock
COPY ./src ./src

RUN cargo build --release

CMD ["./target/release/wsh"]
