CREATE TABLE IF NOT EXISTS transfer (
    "amount" String,
    "source" String,
    "destination" String,
    "signer" String,
    "evt_tx" String,
    "evt_block_timestamp" TIMESTAMP,
    "evt_block_height" UInt64,
    "evt_block_hash" String,
    "evt_instruction_index" UInt32
) ENGINE = MergeTree PRIMARY KEY ("evt_tx","evt_instruction_index");
