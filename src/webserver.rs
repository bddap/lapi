use crate::common::*;
use futures::Future;
use warp::{filters::body::json as filter_json, path, post, Filter};

pub fn serve() -> Result<(), ServeError> {
    let api_low = ApiLow {
        database: FakeDb::new(),
        lighting_node: init_default_lightning_client().map_err(ServeError::Create)?,
    };
    let api = ApiHigh {
        api_low,
        log: FakeLog,
    };

    // POST
    // /invoice
    // {
    //   "lesser": "<hex u256>",
    //   "satoshis": <integer>
    // }
    // -> { "error": { "to_large": null } }
    //  | { "ok": {
    //      "invoice": "<bech32 invoice>",
    //      "extras": {
    //        "qr": "https://bechtoqr.com/<bech32 invoice>/qr.png",
    //        "payment_hash": "<hex u256>"
    //      }
    //    }}
    // let post_invoice = post(path("invoice"))
    //     .and(filter_json())
    //     .map(|req: GenerateInvoiceRequest| api.generate_invoice(req));

    // POST
    // /pay
    // {
    //   "master": "<hex u256>",
    //   "invoice": "<bech32 invoice>",
    //   "fee_satoshis": <uint>
    // }
    // -> { "error": { "not_divisible": null }
    //             | { "overflow": null }
    //             | { "insufficient_balance": null }
    //             | { "no_balance": null }
    //    }
    //  | { "ok": { "fees_paid_satoshis": <uint> } }

    // GET
    // /balance/<middle: hex u256>
    // -> { "error": { "no_balance": null } }
    //  | { "ok": { "balance_satoshis": <uint> } }

    // GET
    // /invoice/<payment hash: hex u256>
    // -> { "error": { "expired": null } | { "non_existent": null } }
    //  | { "ok": { "waiting": null }
    //          | { "preimage": "<hex u256>" }
    //    }

    // GET
    // Upgrade: websocket
    // /invoice/<payment hash: hex u256>
    // -> { "error": { "expired": null } | { "non_existent": null } }
    //  | { "ok": { "preimage": "<hex u256>" } }

    // let handler = warp::post2().and(generate_invoice);

    Ok(())
}

#[derive(Debug)]
pub enum ServeError {
    Create(CreateError),
}
