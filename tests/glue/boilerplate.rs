extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate toml;

mod config {
    #![allow(unused)]

    include!(concat!(env!("OUT_DIR"), "/expected_outputs/", test_name!(), "-config.rs"));
}
