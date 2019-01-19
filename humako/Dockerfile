FROM rust:1.31.1-stretch as builder

COPY ./ /home/app
WORKDIR /home/app

RUN cargo build

FROM postgres:11
RUN apt update && apt install mdbtools -y
COPY --from=builder /home/app/target/debug/humako /

CMD /humako


