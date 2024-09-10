extern crate configure_me;

mod config {
    #![allow(unused)]

    include!(concat!(env!("OUT_DIR"), "/expected_outputs/", test_name!(), "-config.rs"));
}
