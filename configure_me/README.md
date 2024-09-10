Configure me
============

A Rust library for processing application configuration easily

About
-----

This crate aims to help with reading configuration of application from files, environment variables and command line arguments, merging it together and validating.
It auto-generates most of the parsing and deserializing code for you based on specification file.
It creates a struct for you, which you can use to read configuration into.
It will contain all the parsed and validated fields, so you can access the information quickly easily and idiomatically.

The generated code is formatted to be easy to read and understand.

Wait a second, why this crate doesn't use derive?
-------------------------------------------------

I'd love to use derive. Unfortunately it doesn't compose well with man page generation and other tooling.

For a longer version, see [docs/why\_not\_derive.md](docs/why_not_derive.md)

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
# This text will be used in the documentation (help etc.)
# It's not mandatory, but your progam will be ugly without it.
doc = "Port to listen on."

[[param]]
name = "bind_addr"
type = "::std::net::Ipv4Addr" # Yes, this works and you can use your own types as well! (impl. Deserialize and ParseArg)
default = "::std::net::Ipv4Addr::new(0, 0, 0, 0)" # Rust expression that creates the value.
doc = "IP address to bind to."

[[param]]
name = "tls_cert"
type = "String"
doc = "Path to the TLS certificate. The connections will be unsecure if it isn't provided."
# optional = true is the default, no need to add it here.
```

Then, create a simple `build.rs` script, or use `unstable-metabuild` on nightly (see below).

```rust
fn main() -> Result<(), configure_me_codegen::Error> {
    configure_me_codegen::build_script_auto()
}
```

*Tip: use [`cfg_me`](https://github.com/Kixunil/cfg_me) to generate a man page for your program.*

Add dependencies to `Cargo.toml`:

```toml
[package]
# ...
build = "build.rs"

# This tells auto build script and other tools where to look for your specification.
[package.metadata.configure_me]
spec = "config_spec.toml"

[dependencies]
configure_me = "0.4.0"

[build-dependencies]
configure_me_codegen = "0.4.7"
```

And finally add appropriate incantations into `src/main.rs`:

```rust
include_config!();

fn main() {
    // Don't worry, unwrap_or_exit() prints a nice message instead of ugly panic.
    let (server_config, _remaining_args) = Config::including_optional_config_files(&["/etc/my_awesome_server/server.conf"]).unwrap_or_exit();

    // Your code here, for example:
    let listener = std::net::TcpListener::bind((server_config.bind_addr, server_config.port)).expect("Failed to bind socket");
}
```

If you need to generate different files for multiple binaries, create a separate file for each binary and then define them separately in `Cargo.toml`:

```toml
# config for binary foo
[package.metadata.configure_me.bin]
foo = "foo_config_spec.toml"

# config for binary bar
[package.metadata.configure_me.bin]
bar = "bar_config_spec.toml"
```

And include the file in `foo` like this:

```rust
include_config!("foo");
```

This needs to be specific because there's no way to detect binary name.

Metabuild feature
-----------------

If you use nightly you can avoid writing the build script by using metabuild instead.
First inform yourself about metabuild status in the [tracking issue](https://github.com/rust-lang/rust/issues/49803).
If you decided to try it out:

0. Remove (or rename) `build.rs`.
1. Add `cargo-features = ["metabuild"]` at the top of your `Cargo.toml` (above `[package]` section).
2. Add `metabuild = ["configure_me_codegen"]` to `[package]` section of your `Cargo.toml`.
3. Add `features = ["unstable-metabuild"]` to `configure_me_codegen` build dependency.
4. Try building your project - it should work

**Important: due to the nature of nightly features there are NO stability guarantees!**
Any changes to the feature will ignore semver.
If you want to get this stable soon test it in your project and report your experience in the tracking issue.
Perhaps help with the Cargo code as well if needed.

Manual page generation
----------------------

The crate exports an interface for generating manual pages, but I recommend you to not worry about it.
There's a [tool](https://github.com/Kixunil/cfg_me) for generating extra files (currently only man page) from your specification. You can install it using `cargo`.

After installing it, you can type `cfg_me man` to see the generated man page. Run `cfg_me -o program_name.1 man` to save it to a file.

Debconf generation
------------------

This crate also contains experimental debconf support behind `debconf` feature. It generates `templates`, `configure` and `postinst` files for you. If you plan to package your application, you can use it. Note that this isn't integrated with `cargo-deb` yet.

In order to use this feature, you must enable the flag in `Cargo.toml`:

```toml
configure_me_codegen = { version = "0.4.0", features = ["debconf"] }
```

Then add debconf options to your configuration specification:

```toml
[debconf]
# Sets the name of the package.
# Enables debconf support.
package_name = "my-awesome-app"

[[param]]
name = "port"
type = "u16"
optional = false
# Documentation IS mandatory for debconf!
doc = "Port to listen on."
# Priority used for debconf questions "high" is recommended for non-default,
# mandatory questions. In case of missing priority, the option is skipped!
debconf_priority = "high"

[[param]]
name = "bind_addr"
type = "::std::net::Ipv4Addr"
# Rust expression that creates the default value.
default = "::std::net::Ipv4Addr::new(0, 0, 0, 0)"
doc = "IP address to bind to."
debconf_priority = "low"
# The default set by debconf.While it might seem redundant, this way the user
# sees the default value when editing.
debconf_default = "0.0.0.0"

[[param]]
name = "tls_cert"
type = "String"
doc = "Path to the TLS certificate. The connections will be unsecure if it isn't provided."
debconf_priority = "medium"
```

Finally build your application with `DEBCONF_OUT` environment variable set to existing directory
where `configure_me` should generate the files.

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

On the other hand, it's much more mature and supports some features this crate doesn't (bash completion and native subcommands).

`clap` may be more suitable for programs that should be easy to work with from command line, `configure_me` may be better for long-running processes with a lot of configuration options.

License
-------

MITNFA
