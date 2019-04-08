use crate::common::*;
use lightning_invoice::ParseOrSemanticError;

pub trait Log: Sync + Send {
    fn _err(&self, err: LogErr);

    fn err(&self, err: LogErr) -> ErrLogged {
        self._err(err);
        ErrLogged(Private)
    }
}

#[derive(Debug, Clone)]
pub enum LogErr {
    InvoiceCreateNetwork {
        backend_name: String,
        err: String,
    },
    /// ParseOrSemanticError is not currently [de]serializable.
    /// https://github.com/rust-bitcoin/rust-lightning-invoice/issues/30
    InvalidInvoiceCreated(ParseOrSemanticError),
    DbStoreInvoiceDuplicate(Lesser, Invoice),
    /// Invoice was paid, but the hash(preimage) != payment_hash backends such as lnd are expected
    /// to guard against this.
    PayPreimageNoMatch {
        outgoing_paid_invoice: PaidInvoiceOutgoing,
    },
    PayInvoiceOverflowOnRefund(DepositError),
    /// Refunding change to account after payment would cause an overflow.
    /// This error is nigh impossible to trigger via legitimate means.
    PayInvoiceOverflowOnRefundFee(DepositError),
    PayAmountTooLarge {
        amount: Satoshis,
    },
    PayFeeTooLarge {
        fee: Fee<Satoshis>,
    },
    /// TODO this variant should be removed before prod
    PayUnknownError(String),
}

/// This type is not constructable outside this file.
/// It is empty, but serves as proof that an error was logged.
/// Clone is intentionally not implemented becase only one
/// ErrLogged instance should exist per call to log.
#[derive(Debug)]
pub struct ErrLogged(Private);
#[derive(Debug)]
struct Private;

#[derive(Debug)]
pub enum LoggedOr<T> {
    Logged(ErrLogged),
    UnLogged(T),
}

/// Errors may implement this trait when it's possible they are an internal
/// error. Internal errors indicate a problem with the api server, and should
/// not be sent to api clients. Instead, internal errors are logged.
/// In the case of an http REST request, internal errors are logged, then a
/// 500 is sent.
pub trait MaybeServerError {
    /// The type returned when the error is not logged. This type will be sent
    /// to api clients.
    type NotServerError;

    fn maybe_log<L: Log>(self, log: &L) -> LoggedOr<Self::NotServerError>
    where
        Self: std::marker::Sized,
    {
        match self.try_as_response() {
            Ok(response) => LoggedOr::UnLogged(response),
            Err(err) => LoggedOr::Logged(log.err(err)),
        }
    }

    fn try_as_response(self) -> Result<Self::NotServerError, LogErr>;
}

pub trait ServerError: Sized {
    fn into_log_err(self) -> LogErr;

    fn log<L: Log>(self, log: &L) -> ErrLogged {
        log.err(self.into_log_err())
    }
}
