export RUST_LOG=info
../target/release/nockchain --fakenet --genesis-leader --npc-socket nockchain.sock --mining-pubkey "3oSuHZHPFtioRAxzGDgC4xL6XZBETXKU5uLkEJfZk12kkiph5JckPT2HQYaqwA8PigkhvwKv5nDEs8wLaX51o5Pr9x3aL746HH5YaDsV3S5nubiEMbVsHJFByKXGcFAATzVb" --bind /ip4/0.0.0.0/udp/3005/quic-v1 --peer /ip4/127.0.0.1/udp/3006/quic-v1 --new-peer-id --no-default-peers --mine
