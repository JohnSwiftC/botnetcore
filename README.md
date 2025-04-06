# A Simple Rust Botnet

Built this for fun and wrote a small article about it. It was sitting as two seperate repos and I never got around to merging them into one and publishing it, but here we are.

# Features

- RSA Message Signing from the C2 server.
- Execute system commands on all machines.
- Basic multithreaded DDoS functionality,
- Basic persistence on Windows machines.

# Usage

This is really only made to be used on Windows machines, as most of the features are integrated for that OS.

Please ensure you generate your own keypair if you want to use this. The keypair included in the repo should not be used in practice for obvious reasons.

The clients will attempt to ping the C2 server for a message. The message sent will be the first argument to the C2 server in the command line, something like `cargo run -- 'cmd echo hello` will broadcast the `echo hello` command to all machines.

# Warning

This is obviously illegal to use on unknowing machines, all features of it. This was made for educational purposes and comes with an article on my blog explaining the process.
