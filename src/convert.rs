/// From and Into definitions for crate Types
/// MaybeServerError definitions for crate Types
use crate::common::*;
use url::Url;

impl From<Invoice> for GenerateInvoiceOk {
    fn from(invoice: Invoice) -> GenerateInvoiceOk {
        let bolt11: String = to_bolt11(&invoice);
        let payment_hash = get_payment_hash(&invoice);
        GenerateInvoiceOk {
            invoice: InvoiceSerDe(invoice),
            extras: GenerateInvoiceExtras {
                qr: UrlSerDe(
                    Url::parse(&format!("https://bech32.thum.pw/{}/qr.png", bolt11)).unwrap(),
                ),
                payment_hash,
            },
        }
    }
}

impl MaybeServerError for PayInvoiceError {
    type NotServerError = PayInvoiceErr;
    fn maybe_log<L: Log>(self, log: &L) -> LoggedOr<Self::NotServerError> {
        match self {
            PayInvoiceError::NoAmount => PayInvoiceErr::NoAmount(()).into(),
            PayInvoiceError::NotDivisible => PayInvoiceErr::NotDivisible(()).into(),
            PayInvoiceError::OverFlow {
                unpaid_invoice,
                unpaid_amount,
                unpaid_fee,
            } => PayInvoiceErr::Overflow(()).into(),
            PayInvoiceError::Begin(BeginWithdrawalError::InsufficeintBalance) => {
                PayInvoiceErr::InsufficientBalance(()).into()
            }
            PayInvoiceError::Begin(BeginWithdrawalError::NoBalance) => {
                PayInvoiceErr::NoBalance(()).into()
            }
            PayInvoiceError::Pay(pay_err) => MaybeServerError::maybe_log(pay_err, log),
            PayInvoiceError::Finish(finish_withdrawal_error) => {
                LoggedOr::Logged(log.err(LogErr::FinishWithdrawalError(finish_withdrawal_error)))
            }
        }
    }
}

impl MaybeServerError for PayError {
    type NotServerError = PayInvoiceErr;
    fn maybe_log<L: Log>(self, log: &L) -> LoggedOr<Self::NotServerError> {
        match self {
            PayError::AmountTooLarge => PayInvoiceErr::AmountTooLarge(()).into(),
            PayError::FeeTooLarge => PayInvoiceErr::FeeTooLarge(()).into(),
            PayError::PreimageNoMatch {
                outgoing_paid_invoice,
            } => LoggedOr::Logged(log.err(LogErr::PayPreimageNoMatch {
                outgoing_paid_invoice,
            })),
            PayError::Unknown(unknown) => {
                LoggedOr::Logged(log.err(LogErr::InvoicePayUnknown(unknown)))
            }
        }
    }
}

impl<T> From<T> for LoggedOr<T> {
    fn from(other: T) -> Self {
        LoggedOr::UnLogged(other)
    }
}

impl MaybeServerError for GenerateInvoiceError {
    type NotServerError = GenerateInvoiceErr;
    fn maybe_log<L: Log>(self, log: &L) -> LoggedOr<Self::NotServerError> {
        match self {
            GenerateInvoiceError::Create(create) => MaybeServerError::maybe_log(create, log),
            GenerateInvoiceError::Store(store) => LoggedOr::Logged(ServerError::log(store, log)),
        }
    }
}

impl From<PaidInvoiceOutgoing> for PayInvoiceOk {
    fn from(other: PaidInvoiceOutgoing) -> Self {
        let PaidInvoiceOutgoing {
            paid_invoice:
                PaidInvoice {
                    invoice: _invoice,
                    preimage,
                    amount_paid,
                },
            fees_offered,
            fees_paid,
        } = other;
        PayInvoiceOk {
            preimage,
            fees_paid_satoshis: fees_paid,
        }
    }
}

impl MaybeServerError for CheckBalanceError {
    type NotServerError = CheckBalanceErr;
    fn maybe_log<L: Log>(self, _log: &L) -> LoggedOr<Self::NotServerError> {
        match self {
            CheckBalanceError::NoBalance => LoggedOr::UnLogged(CheckBalanceErr::NoBalance(())),
        }
    }
}

impl From<InvoiceStatus> for CheckInvoiceOk {
    fn from(other: InvoiceStatus) -> Self {
        match other {
            InvoiceStatus::Paid(PaidInvoice {
                invoice,
                preimage,
                amount_paid,
            }) => CheckInvoiceOk::Paid {
                preimage,
                amount_paid_satoshis: amount_paid,
            },
            InvoiceStatus::Unpaid => CheckInvoiceOk::Waiting(()),
        }
    }
}

impl MaybeServerError for CheckInvoiceStatusError {
    type NotServerError = CheckInvoiceErr;
    fn maybe_log<L: Log>(self, log: &L) -> LoggedOr<Self::NotServerError> {
        match self {
            CheckInvoiceStatusError::InvoiceDoesNotExist => CheckInvoiceErr::NonExistent(()).into(),
            CheckInvoiceStatusError::Unknown(string) => {
                LoggedOr::Logged(log.err(LogErr::CheckInvoiceStatusUnknown(string)))
            }
        }
    }
}
