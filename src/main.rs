mod api_highlevel;
mod api_lowlevel;
mod api_types;
mod auth;
mod common;
mod convert;
mod db;
mod fake_db;
mod fake_lighting_node;
mod fake_log;
mod invoice;
mod lighting_node;
mod lnd_client;
mod log;
mod preimage;
mod satoshis;
mod semantics;
mod ser_de;
mod u256;
mod webserver;

fn main() {
    webserver::serve().unwrap()
}
