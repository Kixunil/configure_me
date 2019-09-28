Configure me
============

A Rust library for processing application configuration easily

About
-----

This crate aims to help with reading configuration of application from files, environment variables and command line arguments, merging it together and validating.
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

First create `config_spec.toml` configuration file specifying all the parameters:

```toml
[[param]]
name = "port"
type = "u16"
optional = false
# This text will be used in the documentation (help etc)
# It's not mandatory, but your progam will be ugly without it.
doc = "Port to listen on."

[[param]]
name = "bind_addr"
type = "::std::net::Ipv4Addr" # Yes, this works and  you can use your own types implementing Deserialize and ParseArg as well!
default = "::std::net::Ipv4Addr::new(0, 0, 0, 0)" # Rust expression that creates the value
doc = "IP address to bind to."

[[param]]
name = "tls_cert"
type = "String"
doc = "Path to the TLS certificate. The connections will be unsecure if it isn't provided."
# optional = true is the default, no need to add it here
```

Then, create a simple `build.rs` script like:

```rust
extern crate configure_me_codegen;

fn main() {
    configure_me_codegen::build_script("config_spec.toml").unwrap();
}
```

*Tip: use `configure_me::build_script_with_man` to generate man page as well.*

Add dependencies to `Cargo.toml`:

```toml
[package]
# ...
build = "build.rs"

[dependencies]
configure_me = "0.3.3"

[build-dependencies]
configure_me_codegen = "0.3.8"
```

And finally add appropriate incantiations into `src/main.rs`:

```rust
#[macro_use]
extern crate configure_me;

include_config!();

fn main() {
    let (server_config, _remaining_args) = Config::including_optional_config_files(&["/etc/my_awesome_server/server.conf"]).unwrap_or_exit();

    // Your code here
    // E.g.:
    let listener = std::net::TcpListener::bind((server_config.bind_addr, server_config.port)).expect("Failed to bind socket");
}
```

Planned features
----------------

This crate is unfinished and there are features I definitelly want:

* Support for documenting your configuration very well - done
* Support environment variables - done
* Generate bash completion
* Some advanced features

Comparison with clap
--------------------

`clap` is a great crate that works well. Unfortunately, it doesn't support reading from config files. It also has stringly-typed API, which adds boilerplate and (arguably small, but non-zero) runtime overhead.

On the other hand, it's much more mature and supports some features, this crate doesn't (bash completion and native subcommands).

`clap` may be more suitable for programs that should be easy to work with from command line, `configure_me` may be better for long-running processes with a lot of configuration options.

License
-------

MITNFA
