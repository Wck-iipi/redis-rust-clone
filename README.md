# redis-rust-clone
This is a redis clone(currently WIP). It implements the following features of redis:
1. It support redis arrays, numbers, simpleString, bulkstring.
2. Supports ping, SET and GET(Along with expiry), INFO and replicaof.

## How to run:
1. Install this with `cargo build`.
2. Run this server with `spawn_redis_server.sh`.
3. Run the commands with `nc 127.0.0.1 6379` and then adding the required commands.
