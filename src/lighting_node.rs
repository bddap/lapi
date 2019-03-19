use crate::common::*;
use futures::future::FutureResult;

pub trait LightningNode {
    /// Generate a unique invoice for n satoshis.
    fn create_invoice(&self, satoshis: Satoshis) -> FutureResult<Invoice, CreateInvoiceError>;

    /// Send to invoice. If invoice does not specfy an amount, return a PayError.
    fn pay_invoice(&self, invoice: Invoice) -> FutureResult<PaidInvoice, PayError>;
}

pub enum CreateInvoiceError {
    /// Payment amount exeeded 2^64-1 pico-btc.
    TooLarge,
}

pub enum PayError {
    NoAmount,
}
