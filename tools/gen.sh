#!/bin/bash

set -eu

IDS_URL="https://pci-ids.ucw.cz/v2.2/pci.ids"

WORKDIR=`mktemp -d`
trap 'rm -rf ${WORKDIR}' EXIT

# Generate binding.
curl -sSL -o ${WORKDIR}/pci.ids ${IDS_URL}

# Generate code.
cargo run --example ids-gen ${WORKDIR}/pci.ids > src/ids.rs
rustfmt src/ids.rs
