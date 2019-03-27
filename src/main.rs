mod api_types;
mod auth;
mod common;
mod db;
mod endpoints;
mod fake_db;
mod fake_lighting_node;
mod invoice;
mod lighting_node;
mod lnd_client;
mod satoshis;
mod semantics;
mod ser_de;
mod u256;
mod webserver;

fn main() {
    webserver::serve().unwrap()
}
