FROM golang:1.11-alpine as builder

MAINTAINER Andrew Dirksen <andrew.dirksen@layer1.capital>

# Force Go to use the cgo based DNS resolver. This is required to ensure DNS
# queries required to connect to linked containers succeed.
# Note from Andrew, I do not understand what this is for. I'm cargo-culting here.
ENV GODEBUG netdns=cgo

# Install lnd build deps.
RUN apk add --update alpine-sdk \
    git \
    make

# Get lnd repo.
RUN mkdir -p /go/src/github.com/lightningnetwork \
&&  cd /go/src/github.com/lightningnetwork \
&&  git clone https://github.com/lightningnetwork/lnd \
&&  cd lnd \
&&  git checkout 9f6a1403e9773ba97ec0e9fdee3b654789970d3a

# Build lnd.
RUN cd /go/src/github.com/lightningnetwork/lnd \
&&  make \
&&  make install

# Start a new, final image to reduce size.
FROM alpine as final

# Expose lnd ports (server, rpc).
EXPOSE 9735 10009

# Copy the binaries and entrypoint from the builder image.
COPY --from=builder /go/bin/lncli /bin/
COPY --from=builder /go/bin/lnd /bin/

# Add bash.
RUN /bin/sh -c "apk add --no-cache bash"

# Copy the entrypoint script.
COPY "start-lnd.sh" .
RUN chmod +x start-lnd.sh
