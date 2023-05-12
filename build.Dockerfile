FROM ubuntu

RUN apt update && apt install -y libssl-dev ca-certificates catimg && rm -rf /var/lib/apt/lists/*

COPY target/release/catimg-backend /usr/local/bin/catimg-backend
COPY frontend/dist /frontend

ENTRYPOINT ["/usr/local/bin/catimg-backend"]