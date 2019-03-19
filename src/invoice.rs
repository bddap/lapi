use crate::common::*;

pub use lightning_invoice::{Invoice, Sha256};

#[derive(Clone)]
pub enum InvoiceStatus {
    Paid,
    Unpaid,
}

pub struct PaidInvoice(pub Invoice);

/// Serves as a UUID for an invoice. Used to associate an invoice with a Lesser.
#[derive(PartialOrd, Ord, PartialEq, Eq, Clone)]
pub struct PaymentHash(u256::U256);

pub fn invoice_uuid(invoice: &Invoice) -> PaymentHash {
    invoice.payment_hash().clone().into()
}

impl From<Sha256> for PaymentHash {
    fn from(other: Sha256) -> PaymentHash {
        unimplemented!()
    }
}
