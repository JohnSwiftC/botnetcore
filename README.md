# A Simple Rust Botnet

Built this for fun and wrote a small article about it. It was sitting as two seperate repos and I never got around to merging them into one and publishing it, but here we are.

# Important Note

While this was at one time inteneded to be multi-platform, it is now solely focused on Windows machine for the client program. The server can of course still be ran anywhere.

# Features

- RSA Message Signing from the C2 server.
- Execute system commands on all machines.
- Basic multithreaded DDoS functionality,
- Basic persistence on Windows machines.

# Usage

For the server, set the following environment variables:

- `BOTNET_LISTENIP` - Your listen address with the port included, will most likely be 0.0.0.0:8000, with a different port.

- `BOTNET_PRIVATEPEM` - The private pkcs1 pem of your keypair. The public key must be manually included in the build of the client executable for communication between the two to succeed.

For the client, manually set the connect back IP with the port, as well as the public pem of your keypair.

This is really only made to be used on Windows machines, as most of the features are integrated for that OS.

Please ensure you generate your own keypair if you want to use this. The keypair included in the repo should not be used in practice for obvious reasons.

The clients will attempt to ping the C2 server for a message. The message sent will be the first argument to the C2 server in the command line, something like `cargo run -- 'cmd echo hello` will broadcast the `echo hello` command to all machines.

# Commands

- `cmd <command>` - Runs a shell command on all machines in the network.

- `httpddos <url> <duration in secs>` - Spawns threads on machines in the network and spams the target with a large amount of requests. 

# Warning

This is obviously illegal to use on unknowing machines, all features of it. This was made for educational purposes and comes with an article on my blog explaining the process.
