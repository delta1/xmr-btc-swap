# syntax=docker/dockerfile:1

# build
FROM rust:1.59-slim-bullseye AS builder

RUN update-ca-certificates
RUN apt update && apt install -y wget autoconf pkg-config make

ENV USER=asb
ENV UID=10001

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/home/asb" \
    --shell "/bin/bash" \
    --uid "${UID}" \
    "${USER}"

WORKDIR /usr/local/src
RUN wget https://github.com/jpmens/jo/releases/download/1.6/jo-1.6.tar.gz
RUN tar -avxf jo-1.6.tar.gz

WORKDIR /usr/local/src/jo-1.6
RUN autoreconf -i
RUN ./configure
RUN make && make install

WORKDIR /asb

COPY .git/ .git
COPY monero-harness/ monero-harness
COPY monero-rpc/ monero-rpc
COPY monero-wallet/ monero-wallet
COPY swap/ swap
COPY Cargo.toml .
COPY Cargo.lock .

RUN cargo build --release --locked --package swap --bin asb

# final container
FROM debian:bullseye-slim

COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group
COPY --from=builder /home/asb /home/asb

RUN mkdir -p /etc/asb
RUN chown -R asb:asb /etc/asb

USER asb:asb

COPY --from=builder /usr/local/bin/jo /usr/local/bin
COPY --from=builder /asb/target/release/asb /usr/local/bin
COPY docker/asb.sh /usr/local/bin
# COPY config.json /etc/asb/

ENTRYPOINT ["/usr/local/bin/asb.sh"]
CMD ["/usr/local/bin/asb"]
