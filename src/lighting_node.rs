use crate::common::*;
use futures::future::FutureResult;

pub trait LightningNode {
    /// Generate a unique invoice for n satoshis.
    fn create_invoice(
        &self,
        lesser: Lesser,
        satoshis: Satoshis,
    ) -> FutureResult<Invoice, CreateInvoiceError>;

    fn pay_invoice(&self, invoice: Invoice) -> FutureResult<PaidInvoice, PayError>;
}

pub enum CreateInvoiceError {}

pub enum PayError {}
