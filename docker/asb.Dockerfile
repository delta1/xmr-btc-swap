# syntax=docker/dockerfile:1

# build
FROM rust:1.62-slim-bullseye AS builder

RUN update-ca-certificates
RUN apt-get update && apt-get install -y wget autoconf pkg-config make gpg git tor

ENV USER=asb
ENV UID=10001

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/home/asb" \
    --shell "/bin/bash" \
    --uid "${UID}" \
    "${USER}"
RUN usermod -aG debian-tor asb

WORKDIR /tmp
RUN wget https://raw.githubusercontent.com/monero-project/monero/master/utils/gpg_keys/binaryfate.asc
RUN gpg --import binaryfate.asc
RUN wget https://www.getmonero.org/downloads/hashes.txt
RUN gpg --verify hashes.txt
RUN wget https://downloads.getmonero.org/cli/monero-linux-x64-v0.18.1.2.tar.bz2
RUN test "$(grep monero-linux-x64 hashes.txt)" = "$(sha256sum monero-linux-x64-v0.18.1.2.tar.bz2)"
RUN tar -avxf monero-linux-x64-v0.18.1.2.tar.bz2
RUN cp monero-x86_64-linux-gnu-v0.18.1.2/monero-wallet-rpc /usr/local/bin

WORKDIR /asb

COPY .git/ .git
COPY monero-harness/ monero-harness
COPY monero-rpc/ monero-rpc
COPY monero-wallet/ monero-wallet
COPY swap/ swap
COPY Cargo.toml .
COPY Cargo.lock .

RUN rm -rf ~/.cargo || true

ENV CARGO_NET_GIT_FETCH_WITH_CLI=true
RUN cargo fetch --verbose --locked
RUN cargo build --release --locked --package swap --bin asb

# run
FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y tor
RUN service tor start

COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group
COPY --from=builder /home/asb /home/asb

RUN mkdir -p /home/asb/data
RUN chown -R asb:asb /home/asb
RUN usermod -aG debian-tor asb

USER asb:asb

COPY --from=builder /usr/local/bin/monero-wallet-rpc /usr/local/bin
COPY --from=builder /asb/target/release/asb /usr/local/bin
COPY docker/config.toml /home/asb/config.toml
COPY docker/asb.sh /usr/local/bin

ENTRYPOINT ["/usr/local/bin/asb.sh"]
CMD ["/usr/local/bin/asb"]
