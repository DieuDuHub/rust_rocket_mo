#FROM debian:buster-slim
#FROM alpine:3.14
#FROM arm64v8/debian
####################################################################################################
## Builder
####################################################################################################
FROM rust:latest AS builder
RUN set -eux; \
    apt-get update; \
    apt-get install -y --no-install-recommends \
        ca-certificates \
        libssl-dev

RUN rustup target add x86_64-unknown-linux-musl
RUN apt update && apt install -y musl-tools musl-dev
RUN update-ca-certificates

# Create appuser
ENV USER=myip
ENV UID=10001

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    "${USER}"


WORKDIR /rocket_mo

COPY ./ .
COPY ./Rocket.toml .

#RUN cargo build --target x86_64-unknown-linux-musl --release
RUN cargo build --release

#
#USER myip:myip

#CMD ["cd /usr/src/configserver/target/release/"]
#CMD ["./rust-config-server"]

#EXPOSE 8081/tcp

####################################################################################################
## Final image
####################################################################################################
#FROM debian:bullseye
FROM debian:bookworm-slim
RUN set -eux; \
    apt-get update; \
    apt-get install -y --no-install-recommends \
        ca-certificates \
        libssl-dev
# Import from builder.
COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group

WORKDIR /rocket_mo

# Copy our build
COPY --from=builder /rocket_mo/target/release/rocket_mo ./
COPY --from=builder /rocket_mo/wait-for-it.sh ./
COPY --from=builder /rocket_mo/Rocket.toml .

# Use an unprivileged user.
USER myip:myip

COPY ./wait-for-it.sh /rocket_mo/wait-for-it.sh

#CMD ["/rocket_mo/rocket_mo"]

EXPOSE 8000/tcp
