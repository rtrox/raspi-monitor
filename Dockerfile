FROM docker.io/library/rust:1.81-alpine as builder
ARG REVISION
ARG VERSION

RUN apk add --no-cache musl-dev openssl-dev openssl-libs-static pkgconf git

COPY Cargo.toml Cargo.lock /code/
COPY /src/ /code/src/

# force static linking
ENV SYSROOT=/dummy
ENV VERSION=$VERSION
ENV REVISION=$REVISION

# build code
WORKDIR /code
RUN cargo build --bins --release

# runtime container
FROM scratch AS runtime

LABEL org.opencontainers.image.title="raspi-monitor" \
    org.opencontainers.image.description="Rust-Based SSD1306 OLED Raspberry Pi Monitor" \
    org.opencontainers.image.version=$VERSION \
    org.opencontainers.image.revision=$REVISION \
    org.opencontainers.image.authors="Russell Troxel"

ENV RUST_LOG=info

COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
COPY --from=builder /code/target/release/raspi-monitor /usr/local/bin/raspi-monitor

# set entrypoint
ENTRYPOINT ["/usr/local/bin/raspi-monitor"]
