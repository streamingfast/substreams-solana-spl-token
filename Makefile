ENDPOINT ?= mainnet.sol.streamingfast.io:443
START_BLOCK ?= 286135778
STOP_BLOCK ?= +10

.PHONY: build
build:
	LDFLAGS="-Wl,-no_compact_unwind" cargo build --target wasm32-unknown-unknown --release

.PHONY: map_block
map_block: build
	substreams run -e $(ENDPOINT) tokens/substreams.yaml map_block -s $(START_BLOCK) -t $(STOP_BLOCK)

.PHONY: db_out
db_out: build
	substreams run -e $(ENDPOINT) tokens/substreams.yaml db_out -s $(START_BLOCK) -t $(STOP_BLOCK)

.PHONY: protogen
protogen:
	cd tokens && substreams protogen substreams.yaml --exclude-paths="sf/solana/type,sf/substreams,google"

.PHONY: pack
pack:
	cd tokens && substreams pack ./substreams.yaml