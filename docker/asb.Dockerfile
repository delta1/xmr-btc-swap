# syntax=docker/dockerfile:1

# build
FROM rust:1.59-slim-bullseye AS builder

RUN update-ca-certificates
RUN apt update && apt install -y wget autoconf pkg-config make gpg

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

WORKDIR /tmp
RUN wget https://raw.githubusercontent.com/monero-project/monero/master/utils/gpg_keys/binaryfate.asc
RUN gpg --import binaryfate.asc
RUN wget https://www.getmonero.org/downloads/hashes.txt
RUN gpg --verify hashes.txt
RUN wget https://downloads.getmonero.org/cli/monero-linux-x64-v0.17.3.0.tar.bz2
RUN SHASUM=
RUN test "$(grep monero-linux-x64 hashes.txt)" = "$(sha256sum monero-linux-x64-v0.17.3.0.tar.bz2)"
RUN tar -avxf monero-linux-x64-v0.17.3.0.tar.bz2
RUN cp monero-x86_64-linux-gnu-v0.17.3.0/monero-wallet-rpc /usr/local/bin

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

RUN mkdir -p /etc/asb /home/asb/data
RUN chown -R asb:asb /etc/asb /home/asb

USER asb:asb

COPY --from=builder /usr/local/bin/jo /usr/local/bin
COPY --from=builder /usr/local/bin/monero-wallet-rpc /usr/local/bin
COPY --from=builder /asb/target/release/asb /usr/local/bin
COPY docker/asb.sh /usr/local/bin

ENTRYPOINT ["/usr/local/bin/asb.sh"]
CMD ["/usr/local/bin/asb"]
