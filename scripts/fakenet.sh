export RUST_LOG=info
../target/release/nockchain --fakenet --genesis-leader --npc-socket nockchain.sock --mining-pubkey "2sZFDAmx3fuDq4JNgqHiwj3Gp6ysaZ8pYxGyXjeWWSii4yX54B6wpNBiMLAatkKnAMcw6P1Y8LzcgEqZYX4CRDK81ceCzngZD9gu3qSixyxjyJUMFUgNDAv5imJXkbMEYMSy" --bind /ip4/0.0.0.0/udp/3005/quic-v1 --peer /ip4/127.0.0.1/udp/3006/quic-v1 --new-peer-id --no-default-peers --mine
