use crate::common::*;
pub use lightning_invoice::{Invoice, Sha256};
use lightning_invoice::{ParseOrSemanticError, SignedRawInvoice};
use serde::{Deserialize, Serialize, Serializer};
use std::borrow::Borrow;

#[derive(Clone)]
pub enum InvoiceStatus {
    Paid(PaidInvoice),
    Unpaid,
}

#[derive(Debug, Clone)]
pub struct PaidInvoice {
    pub invoice: Invoice,
    // hash(preimage) == invoice.payment_hash()
    pub preimage: Preimage,
    pub amount_paid: Satoshis,
}

#[derive(Debug, Clone)]
pub struct PaidInvoiceOutgoing {
    pub paid_invoice: PaidInvoice,
    pub fees_offered: Fee<Satoshis>,
    pub fees_paid: Fee<Satoshis>,
}

pub fn get_payment_hash(invoice: &Invoice) -> U256 {
    let sl: &[u8] = invoice.payment_hash().0.borrow();
    debug_assert_eq!(sl.len(), 32);
    U256::try_from_slice(sl).unwrap()
}

pub fn parse_bolt11(encoded: &str) -> Result<Invoice, ParseOrSemanticError> {
    let raw = encoded.parse::<SignedRawInvoice>()?;
    let invoice = Invoice::from_signed(raw)?;
    Ok(invoice)
}

pub fn to_bolt11(invoice: &Invoice) -> String {
    invoice.to_string()
}

#[cfg(test)]
mod test {
    fn parse_then_deserialize() {
        let some_invoice =
            "lnbc420n1pwf2rsfpp5cakf9e6fvcreyywflk0p9wekl4whwk6qm2ge05g2vhjl5ae0gj5qdpsd3h8x6pwwpmj\
             qmrfde6hsgrrdah8gctfdejhygrxdaezqvtgxqzfvcqp2rzjq2psxxpvnzza4yankfwfvgwj9ne5ga0x8sfrjs\
             hyq244xrq92mn82zyt6yqqgksqqqqqqqqqqqqqqeqqjq7fxyyw5d63ghg4lau9v5zeuttswjlcsprf44y2rv2p\
             c5ew0wr67kzs27gaycuxhz7eex4l92fywd2k44nw9eck4k6eqh394y3kclqssp7yersm";
        // parse some_invoice
        // deserialize parsed invoice
        // assert deserialized == original
        panic!("test not implemented");
    }

    fn invalid_bolt11() {
        // parse invalid invoice
        // assert fail
        panic!("test not implemented");
    }
}
