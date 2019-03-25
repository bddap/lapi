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
mod u256;

fn main() {
    println!("Hello, world!");
}

// /generate-invoice
// {
//   "lesser": "<hex u256>",
//   "satoshis": <integer>
// }
// -> { "error": { "to_large": null } }
// {
//   "invoice": "<bech32 invoice>",
//   "extras": {
//     "qr": "https://bechtoqr.com/<bech32 invoice>.png",
//     "payment_hash": "<hex u256>"
//   },
// }

// /pay-invoice
// {
//   "greater": "<hex u256>",
//   "invoice": "<bech32 invoice>",
//   "fee_sataoshis": <uint>
// }
// -> { "error": { "not_divisible": null }
//             | { "overflow": null }
//             | { "insufficeint_balance": null }
//             | { "no_balance": null }
//    }
//  | { "ok": { "fees_paid_satoshis": <uint> } }

// /check-balance
// { "middle": "<hex u256>" }
// -> { "error": { "no_balance": null } }
//  | { "ok": { "balance_satoshis": <uint> } }

// /check-invoice
// { "payment_hash": "<hex u256>" }
// -> { "error": { "expired": null } | { "non_existent": null } }
//  | { "ok": { "waiting": null }
//          | { "preimage": "<hex u256>" }
//    }

// Websocket /await-invoice
// { "payment_hash": "<hex u256>" }
// -> { "error": { "expired": null } | { "non_existent": null } }
//  | { "ok": { "preimage": "<hex u256>" } }
