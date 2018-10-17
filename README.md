Configure me
============

A Rust library for processing application configuration easily

About
-----

This library aims to help with reading configuration of application from files, environment variables and command line arguments, merging it together and validating.
It auto-generates most of the parsing and deserializing code for you based on build configuration (heh) file.
It creates a struct for you, which you can use to read configuration into.
It will contain all the parsed and validated fields, so you can access the information quickly easily and idiomatically.

The generated code is formatted to be easy to read and understand.

Example
-------

Let's say, your application needs these parametrs to run:

* Port - this is mandatory
* IP address to bind to - defaults to 0.0.0.0
* Path to TLS certificate - optional, the server will be unsecure if not given

First you create Toml configuration file specifying all the parameters:

```toml
[[param]]
name = "port"
type = "u16"
optional = false

[[param]]
name = "bind_addr"
type = "::std::net::Ipv4Addr" # Yes, this works and  you can use your own types implementing Deserialize and FromStr as well!
default = "::std::net::Ipv4Addr::new(0, 0, 0, 0)" # Rust expression that creates the value

[[param]]
name = "tls_cert"
type = "String"
# optional = true is the default, no need to add it here
```

Then, you create a build script like this:

```rust
extern crate configure_me;

fn main() {
    let mut out: std::path::PathBuf = std::env::var_os("OUT_DIR").unwrap().into();
    out.push("config.rs");
    let config_spec = std::fs::File::open("config.toml").unwrap();
    let config_code = std::fs::File::create(&out).unwrap();
    configure_me::generate_source(config_spec, config_code).unwrap();
    println!("rerun-if-changed=config.toml");
}
```

Add dependencies to `Cargo.toml`:

```toml
[package]
# ...
build = "build.rs"

[dependencies]
serde = "1"
serde_derive = "1"
toml = "0.4"

[build-dependencies]
configure_me = "0.1"
```

Create a module `src/config.rs` for configuration:

```rust
include!(concat!(env!("OUT_DIR"), "/config.rs"));
```

And finally add appropriate incantiations into `src/main.rs`:

```rust
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate toml;

mod config;

fn main() {
    let server_config = config::Config::gather().unwrap();

    // Your code here
    // E.g.:
    let listener = std::net::TcpListener::bind((server_config.bind_addr, server_config.port)).expect("Failed to bind socket");

}
```

Planned features
----------------

This crate is unfinished and there are features I definitelly want:

* Support for documenting your configuration
* Support environment variables
* Generate bash completion
* Some advanced features

Comparison with clap
--------------------

`clap` is a great crate that works well. Unfortunately, it doesn't support reading from config files. It also has stringly-typed API, which adds boilerplate and (arguably small, but non-zero) runtime overhead.

On the other hand, it's much more mature and supports some features, this crate doesn't (mainly documentation, bash completion and subcommands).

`clap` may be more suitable for programs that should be easy to work with from command line, `configure_me` may be better for long-running processes with a lot of configuration options.

License
-------

MITNFA
