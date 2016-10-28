#man
`usage: {} HOST1 PORT1 HOST2 PORT2`

Multiplexer for ubx-raw messages. Expects ubx raw binary data on host1:port1 and host2:port2.
Will output either the stream of port1 or port2 on stdout. Stream1 is the default stream. Will switch to Stream2 if there's a differential fix (solution quality 6).
Will switch back to stream1 if stream2 quality worsens or no data in stream2 for more than 2 seconds.

# build and run

cargo build
cargo run --bin ubx-multiplex

# release
cargo build --release

# ./repeating.sh | nc -l -k 2224
