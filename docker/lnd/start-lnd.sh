#!/usr/bin/env bash

# `set -e` kills the script if a command fails
# `set -u` kills the script if you evaluate an uninitialized variable
# `set -x` prints out every line before it executes
# `set -o pipefail` is like `set -e` but applies to commands inside of pipelines
set -euxo pipefail

exec lnd \
	--rpclisten="0.0.0.0" \
    --bitcoin.active \
    --bitcoin.mainnet \
    --bitcoin.node=bitcoind \
    --debuglevel=debug \
	--bitcoind.rpchost=blockchain \
	--bitcoind.rpcuser=devuser \
	--bitcoind.rpcpass=devpass \
    --bitcoind.zmqpubrawblock=tcp://blockchain:28336 \
	--bitcoind.zmqpubrawtx=tcp://blockchain:28335
