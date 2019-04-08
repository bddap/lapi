//! Wrapper around ApiLow which logs server errors but still returns user facing errors.
//! When ApiHigh methods log an error, they return an ErrLogged. ErrLogged errors should
//! be handled by reporting a server error e.g. http 500.
//! ApiHigh methods accept and return types defined in api_types.

use crate::api_types;
use crate::common::*;
use futures::{
    future::{loop_fn, Loop},
    Future,
};

pub struct ApiHigh<D: Db, L: LightningNode, G: Log> {
    pub api_low: ApiLow<D, L>,
    pub log: G,
}

impl<D: Db, L: LightningNode, G: Log> ApiHigh<D, L, G> {
    pub fn generate_invoice<'a>(
        &'a self,
        request: api_types::GenerateInvoiceRequest,
    ) -> impl Future<Item = api_types::GenerateInvoiceResponse, Error = ErrLogged> + 'a {
        let api_types::GenerateInvoiceRequest { lesser, satoshis } = request;
        self.api_low
            .generate_invoice(lesser, satoshis)
            .map(Into::into) // convert Invoice to GenerateInvoiceOk
            .then(move |res| to_user_result(res, &self.log))
            .map(Into::into) // convert Result<_, _> to ResultSerDe<_, _>
    }

    pub fn pay_invoice<'a>(
        &'a self,
        request: api_types::PayInvoiceRequest,
    ) -> impl Future<Item = api_types::PayInvoiceResponse, Error = ErrLogged> + 'a {
        let api_types::PayInvoiceRequest {
            master,
            invoice,
            amount_satoshis,
            fee_satoshis,
        } = request;
        self.api_low
            .pay_invoice(master, invoice.0, amount_satoshis, fee_satoshis)
            .map(Into::into) // convert PaidInvoice to PayInvoiceOk
            .then(move |res| to_user_result(res, &self.log))
            .map(Into::into) // convert Result<_, _> to ResultSerDe<_, _>
    }

    pub fn check_balance<'a>(
        &'a self,
        middle: Middle,
    ) -> impl Future<Item = api_types::CheckBalanceResponse, Error = ErrLogged> + 'a {
        self.api_low
            .check_balance(middle)
            .map(|balance_satoshis| api_types::CheckBalanceOk { balance_satoshis })
            .then(move |res| to_user_result(res, &self.log))
            .map(Into::into) // convert Result<_, _> to ResultSerDe<_, _>
    }

    pub fn check_invoice_status<'a>(
        &'a self,
        payment_hash: PaymentHash,
    ) -> impl Future<Item = api_types::CheckInvoiceResponse, Error = ErrLogged> + 'a {
        self.api_low
            .check_invoice_status(payment_hash)
            .map(Into::into) // convert InvoiceStatus to CheckInvoiceOk
            .then(move |res| to_user_result(res, &self.log))
            .map(Into::into) // convert Result<_, _> to ResultSerDe<_, _>
    }

    pub fn await_invoice_status<'a>(
        &'a self,
        payment_hash: PaymentHash,
    ) -> impl Future<Item = api_types::AwaitInvoiceResponse, Error = ErrLogged> + 'a {
        // We actively poll now for simplicity. This must change before prod.
        loop_fn((), move |()| {
            self.api_low
                .check_invoice_status(payment_hash)
                .map(|status| match status {
                    InvoiceStatus::Unpaid => Loop::Continue(()),
                    InvoiceStatus::Paid(paid_invoice) => Loop::Break(paid_invoice),
                })
        })
        .map(Into::into) // convert PaidInvoice to AwaitInvoiceOk
        .then(move |res| to_user_result(res, &self.log))
        .map(Into::into) // convert Result<_, _> to ResultSerDe<_, _>
    }
}

/// Extract and log server error from result if result is a server error.
///
/// if result is a server error, log the server error to log and return Err(ErrLogged)
/// if result is a client facing error, return it as Ok(Err(R))
/// if result is ok, return it as Ok(Ok(K))
fn to_user_result<L: Log, K, E: MaybeServerError<NotServerError = R>, R>(
    res: Result<K, E>,
    log: &L,
) -> Result<Result<K, R>, ErrLogged> {
    let reported = res.map_err(|err| MaybeServerError::maybe_log(err, log));
    match reported {
        Ok(ok) => Ok(Ok(ok)),
        Err(LoggedOr::UnLogged(err)) => Ok(Err(err)),
        Err(LoggedOr::Logged(errlogged)) => Err(errlogged),
    }
}
