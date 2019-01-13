FROM rust:1.31.1-stretch as builder

COPY ./ /home/app
WORKDIR /home/app

RUN cargo build --release

FROM postgres:11
COPY --from=builder /home/app/target/release/humako /

CMD /humako


