use crate::common::U256;
pub use lightning_invoice::{Invoice, Sha256};
use std::borrow::Borrow;

#[derive(Clone)]
pub enum InvoiceStatus {
    Paid,
    Unpaid,
}

pub struct PaidInvoice(pub Invoice);

/// Serves as a UUID for an invoice. Used to associate an invoice with a Lesser.
#[derive(PartialOrd, Ord, PartialEq, Eq, Clone)]
pub struct PaymentHash(U256);

pub fn invoice_uuid(invoice: &Invoice) -> PaymentHash {
    invoice.payment_hash().clone().into()
}

impl From<Sha256> for PaymentHash {
    fn from(other: Sha256) -> PaymentHash {
        let sl: &[u8] = other.0.borrow();
        let mut ar: [u8; 32] = [0u8; 32];
        debug_assert!(sl.len() == 32);
        ar.copy_from_slice(sl);
        PaymentHash(ar)
    }
}
