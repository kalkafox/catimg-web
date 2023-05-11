FROM alpine

RUN apk add --no-cache ca-certificates

FROM scratch

COPY --from=0 /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
COPY target/x86_64-unknown-linux-musl/release/catimg-backend /usr/local/bin/catimg-backend

ENTRYPOINT ["/usr/local/bin/catimg-backend"]