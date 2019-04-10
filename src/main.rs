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
mod future;
mod invoice;
mod lighting_node;
mod lnd_client;
mod log;
mod payment_hash;
mod preimage;
mod satoshis;
mod semantics;
mod ser_de;
mod test_util;
mod u256;
mod webserver;

fn main() {
    let result = webserver::serve();
    println!("{:#?}", result);
}
