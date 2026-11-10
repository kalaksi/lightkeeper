# Build context has to contain the release binary as <docker architecture>/lightkeeper-core,
# e.g. amd64/lightkeeper-core or arm64/lightkeeper-core.
FROM debian:13.7-slim
ARG TARGETARCH
LABEL org.opencontainers.image.source="https://github.com/kalaksi/lightkeeper"
LABEL org.opencontainers.image.description="Lightkeeper Core"
LABEL org.opencontainers.image.licenses="GPL-3.0-or-later"

# Data directory (and core.sock) has to be owned by the UID the core runs as. The SSH user connecting from
# Lightkeeper needs access to the socket, so you will probably want to override this with that user's UID/GID.
ENV LIGHTKEEPER_UID="63412"
ENV LIGHTKEEPER_GID="63412"
# App-scoped mounts (same idea as Flatpak): no extra lightkeeper/ segment under these paths.
ENV LIGHTKEEPER_CONFIG_DIR="/config"
ENV LIGHTKEEPER_DATA_DIR="/data"

RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates libdbus-1-3 && \
    rm -rf /var/lib/apt/lists/* && \
    mkdir -p -m 700 /config /data && \
    chown ${LIGHTKEEPER_UID}:${LIGHTKEEPER_GID} /config /data

COPY --chmod=755 ${TARGETARCH}/lightkeeper-core /usr/local/bin/lightkeeper-core

USER ${LIGHTKEEPER_UID}:${LIGHTKEEPER_GID}
VOLUME ["/config", "/data"]
ENTRYPOINT ["/usr/local/bin/lightkeeper-core", "--config-dir", "/config", "--socket-path", "/data/core.sock"]
