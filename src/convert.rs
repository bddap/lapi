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

impl MaybeServerError for PayInvoiceError {
    type NotServerError = api_types::PayInvoiceErr;
    fn maybe_log<L: Log>(self, log: &L) -> LoggedOr<Self::NotServerError> {
        // match self {
        //     PaymentTooLarge => api_types::PayInvoiceErr::AmountTooLarge(()),
        //     Pay(PayError),
        //     Refund(DepositError),
        //     RefundFee(DepositError),

        //     PayInvoiceError::Withdraw(WithdrawalError::InsufficeintBalance) => {
        //         api_types::PayInvoiceErr::InsufficientBalance(()).into()
        //     }
        //     PayInvoiceError::Withdraw(WithdrawalError::NoBalance) => {
        //         api_types::PayInvoiceErr::NoBalance(()).into()
        //     }
        //     PayInvoiceError::Pay(pay_err) => MaybeServerError::maybe_log(pay_err, log),
        //     PayInvoiceError::Finish(finish_withdrawal_error) => {
        //         LoggedOr::Logged(log.err(LogErr::FinishWithdrawalError(finish_withdrawal_error)))
        //     }
        //     DepositError {
        //         account,
        //         current_balance,
        //         deposit_amount,
        //     } => api_types::PayInvoiceErr::Overflow(()).into(),
        // }
        unimplemented!()
    }
}

impl MaybeServerError for PayError {
    type NotServerError = api_types::PayInvoiceErr;
    fn maybe_log<L: Log>(self, log: &L) -> LoggedOr<Self::NotServerError> {
        // match self {
        //     PayError::AmountTooLarge => api_types::PayInvoiceErr::AmountTooLarge(()).into(),
        //     PayError::FeeTooLarge => api_types::PayInvoiceErr::FeeTooLarge(()).into(),
        //     PayError::PreimageNoMatch {
        //         outgoing_paid_invoice,
        //     } => LoggedOr::Logged(log.err(LogErr::PayPreimageNoMatch {
        //         outgoing_paid_invoice,
        //     })),
        //     PayError::Unknown(unknown) => {
        //         LoggedOr::Logged(log.err(LogErr::InvoicePayUnknown(unknown)))
        //     }
        // }
        unimplemented!()
    }
}

impl<T> From<T> for LoggedOr<T> {
    fn from(other: T) -> Self {
        LoggedOr::UnLogged(other)
    }
}

impl MaybeServerError for GenerateInvoiceError {
    type NotServerError = api_types::GenerateInvoiceErr;
    fn maybe_log<L: Log>(self, log: &L) -> LoggedOr<Self::NotServerError> {
        match self {
            GenerateInvoiceError::Create(create) => MaybeServerError::maybe_log(create, log),
            GenerateInvoiceError::Store(store) => LoggedOr::Logged(ServerError::log(store, log)),
        }
    }
}

impl From<PaidInvoiceOutgoing> for api_types::PayInvoiceOk {
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
        api_types::PayInvoiceOk {
            preimage,
            fees_paid_satoshis: fees_paid,
        }
    }
}

impl MaybeServerError for CheckBalanceError {
    type NotServerError = api_types::CheckBalanceErr;
    fn maybe_log<L: Log>(self, _log: &L) -> LoggedOr<Self::NotServerError> {
        match self {
            CheckBalanceError::NoBalance => {
                LoggedOr::UnLogged(api_types::CheckBalanceErr::NoBalance(()))
            }
        }
    }
}

impl From<InvoiceStatus> for api_types::CheckInvoiceOk {
    fn from(other: InvoiceStatus) -> Self {
        match other {
            InvoiceStatus::Paid(PaidInvoice {
                invoice: _,
                preimage,
                amount_paid,
            }) => api_types::CheckInvoiceOk::Paid {
                preimage,
                amount_paid_satoshis: amount_paid,
            },
            InvoiceStatus::Unpaid => api_types::CheckInvoiceOk::Waiting(()),
        }
    }
}

impl MaybeServerError for CheckInvoiceStatusError {
    type NotServerError = api_types::CheckInvoiceErr;
    fn maybe_log<L: Log>(self, log: &L) -> LoggedOr<Self::NotServerError> {
        match self {
            CheckInvoiceStatusError::InvoiceDoesNotExist => {
                api_types::CheckInvoiceErr::NonExistent(()).into()
            }
            CheckInvoiceStatusError::Unknown(string) => {
                LoggedOr::Logged(log.err(LogErr::CheckInvoiceStatusUnknown(string)))
            }
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
