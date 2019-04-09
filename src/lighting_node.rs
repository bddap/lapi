use crate::common::*;
use futures::future::FutureResult;
use lightning_invoice::ParseOrSemanticError;

pub trait LightningNode: Sync + Send {
    /// Generate a unique invoice for n satoshis.
    fn create_invoice(&self, satoshis: Satoshis) -> FutureResult<Invoice, CreateInvoiceError>;

    /// Send to invoice. If invoice does not specfy an amount, return a PayError.
    fn pay_invoice(
        &self,
        invoice: Invoice,
        amount: Satoshis,
        max_fee: Fee<Satoshis>,
    ) -> FutureResult<PaidInvoiceOutgoing, PayError>;
}

#[derive(Debug, Clone)]
pub enum CreateInvoiceError {
    /// Payment amount exeeded what we can handle.
    TooLarge,
    /// Backend specific network error description.
    Network { backend_name: String, err: String },
    /// Backend created an invoice, but it was not valid.
    InvalidInvoice(ParseOrSemanticError),
}

#[derive(Debug, Clone)]
pub enum PayError {
    AmountTooLarge {
        amount: Satoshis,
    },
    FeeTooLarge {
        fee: Fee<Satoshis>,
    },
    PreimageNoMatch {
        outgoing_paid_invoice: PaidInvoiceOutgoing,
    }, // We can probaly assume lnd will never let this happen.
    /// The payment did not succeed. The payment will never be attempted again.
    PaymentAborted,
    Unknown(String), // TODO, enumerate payment failure modes, remove String, remove Unknown variant
}
