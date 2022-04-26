###################################################################################
## Builder
###################################################################################
FROM rust:latest AS builder

RUN rustup target add x86_64-unknown-linux-musl
RUN apt update && apt install -y musl-tools musl-dev upx
RUN update-ca-certificates

# Create appuser
ENV USER=rust
ENV UID=10001

RUN adduser \
  --disabled-password \
  --gecos "" \
  --home "/nonexistent" \
  --shell "/sbin/nologin" \
  --no-create-home \
  --uid "${UID}" \
  "${USER}"


WORKDIR /workdir

COPY ./ .

RUN cargo build --target x86_64-unknown-linux-musl --release
RUN upx --best --lzma target/x86_64-unknown-linux-musl/release/nordea-rate-metrics

###################################################################################
## Final image
###################################################################################
FROM scratch

# Import from builder.
COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group
COPY --from=builder /etc/ssl/certs /etc/ssl/certs


# Copy our build
COPY --from=builder /workdir/target/x86_64-unknown-linux-musl/release/nordea-rate-metrics /

# Use an unprivileged user.
USER rust:rust

CMD ["/nordea-rate-metrics"]