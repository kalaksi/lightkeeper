# Build context has to contain the release binary as <docker architecture>/lightkeeper-core,
# e.g. amd64/lightkeeper-core or arm64/lightkeeper-core.
FROM debian:13.7-slim
ARG TARGETARCH
LABEL org.opencontainers.image.source="https://github.com/kalaksi/lightkeeper"
LABEL org.opencontainers.image.description="Lightkeeper Core"
LABEL org.opencontainers.image.licenses="GPL-3.0-or-later"

# Socket directory has to be owned by the UID the core runs as and the SSH user connecting from Lightkeeper
# needs access to the socket, so you will probably want to override this with the SSH user's UID/GID.
ENV LIGHTKEEPER_UID="63412"
ENV LIGHTKEEPER_GID="63412"
# Data directory will be $XDG_DATA_HOME/lightkeeper.
ENV XDG_DATA_HOME="/data"
# Keep known_hosts and related paths under the mounted config volume (not $HOME/.config).
ENV LIGHTKEEPER_CONFIG_DIR="/config"

RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates libdbus-1-3 && \
    rm -rf /var/lib/apt/lists/* && \
    mkdir -p -m 700 /config /data/lightkeeper /socket && \
    chown ${LIGHTKEEPER_UID}:${LIGHTKEEPER_GID} /config /data/lightkeeper /socket

COPY --chmod=755 ${TARGETARCH}/lightkeeper-core /usr/local/bin/lightkeeper-core

USER ${LIGHTKEEPER_UID}:${LIGHTKEEPER_GID}
VOLUME ["/config", "/data/lightkeeper", "/socket"]
ENTRYPOINT ["/usr/local/bin/lightkeeper-core", "--config-dir", "/config", "--socket-path", "/socket/core.sock"]
