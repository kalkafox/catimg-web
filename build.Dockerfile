FROM ubuntu

COPY target/release/catimg-backend /usr/local/bin/catimg-backend
COPY frontend/dist /frontend

ENTRYPOINT ["/usr/local/bin/catimg-backend"]