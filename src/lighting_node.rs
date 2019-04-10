use crate::common::*;
use futures::future::FutureResult;
use lightning_invoice::ParseOrSemanticError;

pub trait LightningNode: Sync + Send {
    /// Generate a unique invoice for n satoshis.
    fn create_invoice(&self, satoshis: Satoshis) -> DynFut<Invoice, CreateInvoiceError>;

    /// Send to invoice. If invoice does not specfy an amount, return a PayError.
    fn pay_invoice(
        &self,
        invoice: Invoice,
        amount: Satoshis,
        max_fee: Fee<Satoshis>,
    ) -> DynFut<PaidInvoiceOutgoing, PayError>;

    // TODO subscribe to incoming invoices, returns a Stream
}

#[derive(Debug, Clone)]
pub enum CreateInvoiceError {
    /// Backend specific network error description.
    Network { backend_name: String, err: String },
    /// Backend created an invoice, but it was not valid.
    InvalidInvoice(ParseOrSemanticError),
    /// Generic server error
    Unknown(String),
}

#[derive(Debug, Clone)]
pub enum PayError {
    /// The payment did not succeed. The payment will never be attempted again.
    PaymentAborted,
    InvalidResponse(PaidInvoiceInvalid),
    Unknown(String), // TODO, enumerate payment failure modes, remove String, remove Unknown variant
}
