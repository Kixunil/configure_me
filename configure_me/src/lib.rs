//! This crate aims to help with reading configuration of application from files,
//! environment variables and command line arguments, merging it together and
//! validating. It auto-generates most of the code for you based on configuration (heh)
//! file. It creates a struct for you, which contains all the parsed and validated
//! fields, so you can access the information quickly easily and idiomatically.
//!
//! This is currently only a facade for the dependencies, the core of the crate is in
//! `configure_me_codegen` crate.
//!
//! **Important:** In order to use this crate, you need to create a build script using
//! `configure_me_codegen` to generate the code that will use this crate! See the example.
//!
//! Example
//! -------
//! 
//! Let's say, your application needs these parametrs to run:
//! 
//! * Port - this is mandatory
//! * IP address to bind to - defaults to 0.0.0.0
//! * Path to TLS certificate - optional, the server will be unsecure if not given
//! 
//! First create `config_spec.toml` configuration file specifying all the parameters:
//! 
//! ```toml
//! [[param]]
//! name = "port"
//! type = "u16"
//! optional = false
//! # This text will be used in the documentation (help etc)
//! # It's not mandatory, but your progam will be ugly without it.
//! doc = "Port to listen on."
//! 
//! [[param]]
//! name = "bind_addr"
//! # Yes, this works and  you can use your own T: Deserialize + ParseArg as well!
//! type = "::std::net::Ipv4Addr" 
//! default = "::std::net::Ipv4Addr::new(0, 0, 0, 0)" # Rust expression that creates the value
//! doc = "IP address to bind to."
//! 
//! [[param]]
//! name = "tls_cert"
//! type = "::std::path::PathBuf"
//! doc = "Path to the TLS certificate. The connections will be unsecure if it isn't provided."
//! # optional = true is the default, no need to add it here
//! # If the type is optional, it will be represented as Option<T>
//! # e.g. Option<::std::path::PathBuf> in this case.
//! ```
//! 
//! Then, create a simple build script:
//! 
//! ```rust,ignore
//! extern crate configure_me;
//! 
//! fn main() {
//!     configure_me::build_script("config_spec.toml").unwrap();
//! }
//! ```
//! 
//! Add dependencies to `Cargo.toml`:
//! 
//! ```toml
//! [packge]
//! # ...
//! build = "build.rs"
//! 
//! [dependencies]
//! configure_me = "0.3"
//! 
//! [build-dependencies]
//! configure_me_codegen = "0.3"
//! ```
//! 
//! And finally add appropriate incantiations into `src/main.rs`:
//! 
//! ```rust,ignore
//! #[macro_use]
//! extern crate configure_me;
//! 
//! include_config!();
//! 
//! fn main() {
//!     // This will read configuration from "/etc/my_awesome_server/server.conf" file and
//!     // the command-line arguments.
//!     let (server_config, _remaining_args) = Config::including_optional_config_files(&["/etc/my_awesome_server/server.conf]").unwrap_or_exit();
//! 
//!     // Your code here
//!     // E.g.:
//!     let listener = std::net::TcpListener::bind((server_config.bind_addr, server_config.port)).expect("Failed to bind socket");
//! }
//! ```

pub extern crate serde;
pub extern crate toml;
pub extern crate parse_arg;

#[allow(unused_imports)]
#[macro_use]
extern crate serde_derive;
#[doc(hidden)]
pub use serde_derive::*;

#[macro_export]
macro_rules! include_config {
    () => {
        mod config {
            #![allow(unused)]

            include!(concat!(env!("OUT_DIR"), "/configure_me_config.rs"));
        }

        use config::prelude::*;
    }
}
