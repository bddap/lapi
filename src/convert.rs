/// From and Into definitions for crate Types
use crate::common::*;
use url::Url;

impl From<Invoice> for GenerateInvoiceOk {
    fn from(invoice: Invoice) -> GenerateInvoiceOk {
        let bolt11: String = to_bolt11(&invoice);
        let payment_hash = payment_hash(&invoice);
        GenerateInvoiceOk {
            invoice: InvoiceSerDe(invoice),
            extras: GenerateInvoiceExtras {
                qr: UrlSerDe(
                    Url::parse(&format!("https://bech32.thum.pw/{}/qr.png", bolt11)).unwrap(),
                ),
                payment_hash,
            },
        }
    }
}
