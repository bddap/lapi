use crate::api_types;
/// From and Into definitions for crate Types
/// MaybeServerError definitions for crate Types
use crate::common::*;
use url::Url;

impl From<Invoice> for api_types::GenerateInvoiceOk {
    fn from(invoice: Invoice) -> api_types::GenerateInvoiceOk {
        let bolt11: String = to_bolt11(&invoice);
        let payment_hash = get_payment_hash(&invoice);
        api_types::GenerateInvoiceOk {
            invoice: InvoiceSerDe(invoice),
            extras: api_types::GenerateInvoiceExtras {
                qr: UrlSerDe(
                    Url::parse(&format!("https://bech32.thum.pw/{}/qr.png", bolt11)).unwrap(),
                ),
                payment_hash,
            },
        }
    }
}

impl<T> From<T> for LoggedOr<T> {
    fn from(other: T) -> Self {
        LoggedOr::UnLogged(other)
    }
}

impl From<PaidInvoiceOutgoing> for api_types::PayInvoiceOk {
    fn from(other: PaidInvoiceOutgoing) -> Self {
        let PaidInvoiceOutgoing {
            paid_invoice,
            fees_offered: _,
            fees_paid,
        } = other;
        api_types::PayInvoiceOk {
            preimage: paid_invoice.preimage().clone(),
            fees_paid_satoshis: fees_paid,
        }
    }
}

impl From<WithdrawalError> for PayInvoiceError {
    fn from(other: WithdrawalError) -> Self {
        match other {
            WithdrawalError::InsufficeintBalance => PayInvoiceError::InsufficientBalance,
        }
    }
}

impl From<InvoiceStatus> for api_types::CheckInvoiceOk {
    fn from(other: InvoiceStatus) -> Self {
        match other {
            InvoiceStatus::Paid(paid_invoice) => api_types::CheckInvoiceOk::Paid {
                preimage: paid_invoice.preimage().clone(),
                amount_paid_satoshis: paid_invoice.amount_paid().clone(),
            },
            InvoiceStatus::Unpaid(_) => api_types::CheckInvoiceOk::Waiting(()),
        }
    }
}

impl From<PaidInvoice> for api_types::AwaitInvoiceOk {
    fn from(other: PaidInvoice) -> Self {
        api_types::AwaitInvoiceOk {
            preimage: other.preimage().clone(),
            amount_paid_satoshis: other.amount_paid().clone(),
        }
    }
}

impl From<PaidInvoiceInvalid> for PayError {
    fn from(other: PaidInvoiceInvalid) -> Self {
        PayError::InvalidResponse(other)
    }
}

impl MaybeServerError for PayInvoiceError {
    type NotServerError = api_types::PayInvoiceErr;
    fn try_as_response(self) -> Result<Self::NotServerError, LogErr> {
        match self {
            PayInvoiceError::InsufficientBalance => {
                Ok(api_types::PayInvoiceErr::InsufficientBalance(()))
            }
            PayInvoiceError::Pay(payerr) => payerr.try_as_response(),
            PayInvoiceError::RefundFee(deposit_err) => {
                Err(LogErr::PayInvoiceOverflowOnRefundFee(deposit_err))
            }
            PayInvoiceError::Refund(deposit_err) => {
                Err(LogErr::PayInvoiceOverflowOnRefund(deposit_err).into())
            }
        }
    }
}

impl MaybeServerError for PayError {
    type NotServerError = api_types::PayInvoiceErr;
    fn try_as_response(self) -> Result<Self::NotServerError, LogErr> {
        match self {
            PayError::PaymentAborted => Ok(api_types::PayInvoiceErr::Aborted(())),
            otherwise => Err(LogErr::PayError(otherwise)),
        }
    }
}

impl MaybeServerError for GenerateInvoiceError {
    type NotServerError = api_types::GenerateInvoiceErr;
    fn try_as_response(self) -> Result<Self::NotServerError, LogErr> {
        match self {
            GenerateInvoiceError::Create(create) => create.try_as_response(),
            GenerateInvoiceError::Store(store) => Err(store.into_log_err()),
        }
    }
}

impl MaybeServerError for CheckBalanceError {
    type NotServerError = api_types::CheckBalanceErr;
    fn try_as_response(self) -> Result<Self::NotServerError, LogErr> {
        match self {
            CheckBalanceError::NoBalance => Ok(api_types::CheckBalanceErr::NoBalance(())),
        }
    }
}

impl MaybeServerError for CheckInvoiceStatusError {
    type NotServerError = api_types::CheckInvoiceErr;
    fn try_as_response(self) -> Result<Self::NotServerError, LogErr> {
        match self {
            CheckInvoiceStatusError::InvoiceDoesNotExist => {
                Ok(api_types::CheckInvoiceErr::NonExistent(()))
            }
        }
    }
}

impl MaybeServerError for CreateInvoiceError {
    type NotServerError = crate::api_types::GenerateInvoiceErr;
    fn try_as_response(self) -> Result<Self::NotServerError, LogErr> {
        match self {
            CreateInvoiceError::TooLarge => Ok(crate::api_types::GenerateInvoiceErr::ToLarge(())),
            CreateInvoiceError::Network { backend_name, err } => {
                Err(LogErr::InvoiceCreateNetwork { backend_name, err })
            }
            CreateInvoiceError::InvalidInvoice(err) => Err(LogErr::InvalidInvoiceCreated(err)),
        }
    }
}

impl ServerError for StoreInvoiceError {
    fn into_log_err(self) -> LogErr {
        match self {
            StoreInvoiceError::EntryAlreadyExists(lesser, invoice) => {
                LogErr::DbStoreInvoiceDuplicate(lesser, invoice)
            }
        }
    }
}
