use crate::common::*;
use futures::future::FutureResult;
use lightning_invoice::ParseOrSemanticError;

pub trait LightningNode {
    /// Generate a unique invoice for n satoshis.
    fn create_invoice(&self, satoshis: Satoshis) -> FutureResult<Invoice, CreateInvoiceError>;

    /// Send to invoice. If invoice does not specfy an amount, return a PayError.
    fn pay_invoice(
        &self,
        invoice: Invoice,
        max_fee: Fee<Satoshis>,
    ) -> FutureResult<PaidInvoice, PayError>;
}

#[derive(Debug, Clone)]
pub enum CreateInvoiceError {
    /// Payment amount exeeded what we can handle.
    TooLarge,
    /// Backend specific network error description.
    Network(String),
    /// Backend created an invoice, but it was not valid.
    InvalidInvoice(ParseOrSemanticError),
}

impl MaybeServerError for CreateInvoiceError {
    type NotServerError = crate::api_types::GenerateInvoiceErr;
    fn maybe_log<L: Log>(self, log: &L) -> LoggedOr<Self::NotServerError> {
        match self {
            CreateInvoiceError::TooLarge => LoggedOr::UnLogged(GenerateInvoiceErr::ToLarge(())),
            CreateInvoiceError::Network(err) => {
                LoggedOr::log(log, LogErr::InvoiceCreateNetwork(err))
            }
            CreateInvoiceError::InvalidInvoice(err) => {
                LoggedOr::log(log, LogErr::InvalidInvoiceCreated(err))
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum PayError {
    NoAmount,
    /// Payment + Fee was too large to process
    Overflow,
    Network,
    PreimageNoMatch, // We can probaly assume lnd will never let this happen.
    Unknown(String), // TODO, enumerate payment fialure modes, remove String, remove Unknown variant
}
