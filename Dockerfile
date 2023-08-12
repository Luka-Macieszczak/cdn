FROM rust:1.67

WORKDIR /app

COPY . .
RUN apt update && apt upgrade -y && apt install -y protobuf-compiler libprotobuf-dev && apt-get install protobuf-compiler

RUN cargo build --release --bin server

EXPOSE 50051

CMD [ "./target/release/server"]
