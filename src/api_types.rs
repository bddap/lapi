use crate::common::*;
use serde::{Deserialize, Serialize};

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
#[derive(PartialEq, Clone, Serialize, Deserialize, Debug)]
pub struct GenerateInvoiceRequest {
    pub lesser: Lesser,
    pub satoshis: Satoshis,
}

pub type GenerateInvoiceResponse = ResultSerDe<GenerateInvoiceOk, GenerateInvoiceErr>;

#[derive(PartialEq, Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum GenerateInvoiceErr {
    ToLarge(()),
}

#[derive(PartialEq, Clone, Serialize, Deserialize, Debug)]
pub struct GenerateInvoiceOk {
    pub invoice: InvoiceSerDe,
    pub extras: GenerateInvoiceExtras,
    // TODO return preimage
}

#[derive(PartialEq, Clone, Serialize, Deserialize, Debug)]
pub struct GenerateInvoiceExtras {
    pub qr: UrlSerDe,
    pub payment_hash: PaymentHash,
}

// POST
// /pay
// {
//   "master": "<hex u256>",
//   "invoice": "<bech32 invoice>",
//   "amount_satoshis": <uint>,
//   "fee_satoshis": <uint>
// }
// -> { "error": { "overflow": null }
//             | { "insufficient_balance": null }
//             | { "no_balance": null }
//    }
//  | { "ok": { "fees_paid_satoshis": <uint> } }
#[derive(PartialEq, Clone, Serialize, Deserialize, Debug)]
pub struct PayInvoiceRequest {
    pub master: Master,
    pub invoice: InvoiceSerDe,
    pub amount_satoshis: Satoshis,
    pub fee_satoshis: Fee<Satoshis>,
}

pub type PayInvoiceResponse = ResultSerDe<PayInvoiceOk, PayInvoiceErr>;

#[derive(PartialEq, Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum PayInvoiceErr {
    /// Provided invoice did not specify an amount
    AmountTooLarge(()),
    FeeTooLarge(()),
    Overflow(()),
    InsufficientBalance(()),
    NoBalance(()),
}

#[derive(PartialEq, Clone, Serialize, Deserialize, Debug)]
pub struct PayInvoiceOk {
    pub preimage: Preimage,
    pub fees_paid_satoshis: Fee<Satoshis>,
}

// GET
// /balance/<middle: hex u256>
// -> { "error": { "no_balance": null } }
//  | { "ok": { "balance_satoshis": <uint> } }
pub type CheckBalanceResponse = ResultSerDe<CheckBalanceOk, CheckBalanceErr>;

#[derive(PartialEq, Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum CheckBalanceErr {
    NoBalance(()),
}

#[derive(PartialEq, Clone, Serialize, Deserialize, Debug)]
pub struct CheckBalanceOk {
    pub balance_satoshis: Satoshis,
}

// GET
// /invoice/<payment hash: hex u256>
// -> { "error": { "expired": null } | { "non_existent": null } }
//  | { "ok": { "waiting": null }
//          | { "paid": {
//                "preimage": "<hex u256>",
//                "amount_paid_satoshis": <uint>
//            }
//    }
pub type CheckInvoiceResponse = ResultSerDe<CheckInvoiceOk, CheckInvoiceErr>;

#[derive(PartialEq, Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum CheckInvoiceErr {
    Expired(()),
    NonExistent(()),
}

#[derive(PartialEq, Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum CheckInvoiceOk {
    Waiting(()),
    Paid {
        preimage: Preimage,
        amount_paid_satoshis: Satoshis,
    },
}

// GET
// Upgrade: websocket
// /invoice/<payment hash: hex u256>
// -> { "error": { "expired": null } | { "non_existent": null } }
//  | { "ok": {
//      "preimage": "<hex u256>",
//      "amount_paid_satoshis": <uint>
//    } }
pub type AwaitInvoiceResponse = ResultSerDe<AwaitInvoiceOk, CheckInvoiceErr>;

#[derive(PartialEq, Clone, Serialize, Deserialize, Debug)]
pub struct AwaitInvoiceOk {
    pub preimage: U256,
    pub amount_paid_satoshis: Satoshis,
}

#[cfg(test)]
mod test {
    use super::*;
    use core::fmt::Debug;
    use serde::de::DeserializeOwned;
    use serde_json::{from_value, json, to_value, Value};
    use std::cmp::PartialEq;

    const VALID_INVOICE_A: &'static str =
        "lnbc420n1pwf2rsfpp5cakf9e6fvcreyywflk0p9wekl4whwk6qm2ge05g2vhjl5ae0gj5qdpsd3h8x6pwwpmj\
         qmrfde6hsgrrdah8gctfdejhygrxdaezqvtgxqzfvcqp2rzjq2psxxpvnzza4yankfwfvgwj9ne5ga0x8sfrjs\
         hyq244xrq92mn82zyt6yqqgksqqqqqqqqqqqqqqeqqjq7fxyyw5d63ghg4lau9v5zeuttswjlcsprf44y2rv2p\
         c5ew0wr67kzs27gaycuxhz7eex4l92fywd2k44nw9eck4k6eqh394y3kclqssp7yersm";
    const VALID_U256_A: &'static str =
        "92ff2aabcd1e070b435c09c50bbc208a45417335329d8fd89b5de6492405cfe4";
    const TYPED_U256_A: U256 = U256([
        146, 255, 42, 171, 205, 30, 7, 11, 67, 92, 9, 197, 11, 188, 32, 138, 69, 65, 115, 53, 50,
        157, 143, 216, 155, 93, 230, 73, 36, 5, 207, 228,
    ]);

    // check if deserializing jso results in a, and if serializing a results in jso
    fn ser_de_equiv<T: DeserializeOwned + Serialize + PartialEq + Clone + Debug>(jso: Value, a: T) {
        let deserialized = from_value::<T>(jso.clone()).unwrap();
        let serialized = to_value::<T>(a.clone()).unwrap();
        assert_eq!(deserialized, a);
        assert_eq!(serialized, jso);
    }

    #[test]
    fn post_invoice() {
        ser_de_equiv(
            json!({
                "lesser": VALID_U256_A,
                "satoshis": 20
            }),
            GenerateInvoiceRequest {
                lesser: Lesser(TYPED_U256_A),
                satoshis: Satoshis(20),
            },
        );
        ser_de_equiv::<GenerateInvoiceResponse>(
            json!({ "error": { "to_large": null } }),
            Err(GenerateInvoiceErr::ToLarge(())).into(),
        );
        ser_de_equiv::<GenerateInvoiceResponse>(
            json!({
                "ok": {
                    "invoice": VALID_INVOICE_A,
                    "extras": {
                        "qr": "https://bechtoqr.com/ln1fdadsf/qr.png",
                        "payment_hash": VALID_U256_A
                    }
                }
            }),
            Ok(GenerateInvoiceOk {
                invoice: InvoiceSerDe(VALID_INVOICE_A.parse().unwrap()),
                extras: GenerateInvoiceExtras {
                    qr: "https://bechtoqr.com/ln1fdadsf/qr.png".parse().unwrap(),
                    payment_hash: TYPED_U256_A,
                },
            })
            .into(),
        );
        from_value::<GenerateInvoiceExtras>(json!({
            "qr": "https://bechtoqr.com/ln1fdadsf/qr.png",
            "payment_hash": "adfa"
        }))
        .unwrap_err();
        from_value::<GenerateInvoiceExtras>(json!({
            "qr": "immaurl",
            "payment_hash": VALID_U256_A
        }))
        .unwrap_err();
    }

    #[test]
    fn post_pay() {
        ser_de_equiv(
            json!({
                "master": VALID_U256_A,
                "invoice": VALID_INVOICE_A,
                "amount_satoshis": 30,
                "fee_satoshis": 30
            }),
            PayInvoiceRequest {
                master: Master(TYPED_U256_A),
                invoice: InvoiceSerDe(VALID_INVOICE_A.parse().unwrap()),
                amount_satoshis: Satoshis(30),
                fee_satoshis: Fee(Satoshis(30)),
            },
        );
        ser_de_equiv::<PayInvoiceResponse>(
            json!({ "error": { "overflow": null } }),
            Err(PayInvoiceErr::Overflow(())).into(),
        );
        ser_de_equiv::<PayInvoiceResponse>(
            json!({ "error": { "insufficient_balance": null } }),
            Err(PayInvoiceErr::InsufficientBalance(())).into(),
        );
        ser_de_equiv::<PayInvoiceResponse>(
            json!({ "error": { "no_balance": null } }),
            Err(PayInvoiceErr::NoBalance(())).into(),
        );
        ser_de_equiv::<PayInvoiceResponse>(
            json!({ "ok": {
                "fees_paid_satoshis": 10,
                "preimage": VALID_U256_A,
            } }),
            Ok(PayInvoiceOk {
                fees_paid_satoshis: Fee(Satoshis(10)),
                preimage: Preimage(TYPED_U256_A),
            })
            .into(),
        );
        from_value::<PayInvoiceResponse>(json!({ "ok": { "fees_paid_satoshis": -10 } }))
            .unwrap_err();
        from_value::<PayInvoiceResponse>(json!({ "err": "fees_paid_satoshis" })).unwrap_err();
    }

    #[test]
    fn get_balance() {
        ser_de_equiv::<CheckBalanceResponse>(
            json!({ "error": { "no_balance": null } }),
            Err(CheckBalanceErr::NoBalance(())).into(),
        );
        ser_de_equiv::<CheckBalanceResponse>(
            json!({ "ok": { "balance_satoshis": 0 } }),
            Ok(CheckBalanceOk {
                balance_satoshis: Satoshis(0),
            })
            .into(),
        );
    }

    #[test]
    fn get_invoice() {
        ser_de_equiv::<CheckInvoiceResponse>(
            json!({ "error": { "expired": null } }),
            Err(CheckInvoiceErr::Expired(())).into(),
        );
        ser_de_equiv::<CheckInvoiceResponse>(
            json!({ "error": { "non_existent": null } }),
            Err(CheckInvoiceErr::NonExistent(())).into(),
        );
        ser_de_equiv::<CheckInvoiceResponse>(
            json!({ "ok": { "waiting": null } }),
            Ok(CheckInvoiceOk::Waiting(())).into(),
        );
        ser_de_equiv::<CheckInvoiceResponse>(
            json!({ "ok": {
                "paid": {
                    "preimage": VALID_U256_A,
                    "amount_paid_satoshis": 11
                }
            }}),
            Ok(CheckInvoiceOk::Paid {
                preimage: Preimage(TYPED_U256_A),
                amount_paid_satoshis: Satoshis(11),
            })
            .into(),
        );
    }

    #[test]
    fn await_invoice() {
        ser_de_equiv::<AwaitInvoiceResponse>(
            json!({ "error": { "expired": null } }),
            Err(CheckInvoiceErr::Expired(())).into(),
        );
        ser_de_equiv::<AwaitInvoiceResponse>(
            json!({ "error": { "non_existent": null } }),
            Err(CheckInvoiceErr::NonExistent(())).into(),
        );
        ser_de_equiv::<AwaitInvoiceResponse>(
            json!({ "ok": {
                "preimage": VALID_U256_A,
                "amount_paid_satoshis": 2
            } }),
            Ok(AwaitInvoiceOk {
                preimage: TYPED_U256_A,
                amount_paid_satoshis: Satoshis(2),
            })
            .into(),
        );
    }
}
