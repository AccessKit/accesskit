#![feature(unix_socket_abstract)]

#[macro_use] extern crate zbus;

mod atspi;
mod manager;
mod node;

pub use manager::Manager;