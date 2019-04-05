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

impl MaybeServerError for CreateInvoiceError {
    type NotServerError = crate::api_types::GenerateInvoiceErr;
    fn maybe_log<L: Log>(self, log: &L) -> LoggedOr<Self::NotServerError> {
        match self {
            CreateInvoiceError::TooLarge => {
                LoggedOr::UnLogged(crate::api_types::GenerateInvoiceErr::ToLarge(()))
            }
            CreateInvoiceError::Network { backend_name, err } => {
                LoggedOr::log(log, LogErr::InvoiceCreateNetwork { backend_name, err })
            }
            CreateInvoiceError::InvalidInvoice(err) => {
                LoggedOr::log(log, LogErr::InvalidInvoiceCreated(err))
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum PayError {
    AmountTooLarge,
    FeeTooLarge,
    PreimageNoMatch {
        outgoing_paid_invoice: PaidInvoiceOutgoing,
    }, // We can probaly assume lnd will never let this happen.
    Unknown(String), // TODO, enumerate payment failure modes, remove String, remove Unknown variant
}
