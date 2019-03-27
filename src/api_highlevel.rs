//! Wrapper around ApiLow which logs server errors but still returns user facing errors.
//! When ApiHigh methods log an error, they return an ErrLogged. ErrLogged errors should
//! be handled by reporting a server error e.g. http 500.
//! ApiHigh methods accept and return types defined in api_types.

use crate::common::*;
use futures::Future;

pub struct ApiHigh<D: Db, L: LightningNode, G: Log> {
    pub api_low: ApiLow<D, L>,
    pub log: G,
}

impl<D: Db, L: LightningNode, G: Log> ApiHigh<D, L, G> {
    pub fn generate_invoice<'a>(
        &'a self,
        request: GenerateInvoiceRequest,
    ) -> impl Future<Item = GenerateInvoiceResponse, Error = ErrLogged> + 'a {
        let GenerateInvoiceRequest { lesser, satoshis } = request;
        self.api_low
            .generate_invoice(lesser, satoshis)
            .map(Into::into) // convert Invoice to GenerateInvoiceOk
            .then(move |res| to_user_result(res, &self.log))
            .map(Into::into) // convert Result<_, _> to ResultSerDe<_, _>
    }

    // pub fn pay_invoice<'a>(
    //     &'a self,
    //     master: Master,
    //     invoice: Invoice,
    //     fee: Fee<Satoshis>,
    // ) -> impl Future<Item = PaidInvoice, Error = PayInvoiceError> + 'a {
    // }

    // pub fn check_balance<'a>(
    //     &'a self,
    //     middle: Middle,
    // ) -> impl Future<Item = Satoshis, Error = CheckBalanceError> + 'a {
    // }

    // pub fn check_invoice_status<'a>(
    //     &'a self,
    //     payment_hash: U256,
    // ) -> impl Future<Item = InvoiceStatus, Error = CheckInvoiceStatusError> + 'a {
    // }
}

impl MaybeServerError for GenerateInvoiceError {
    type NotServerError = GenerateInvoiceErr;
    fn maybe_log<L: Log>(self, log: &L) -> LoggedOr<Self::NotServerError> {
        match self {
            GenerateInvoiceError::Create(create) => MaybeServerError::maybe_log(create, log),
            GenerateInvoiceError::Store(store) => ServerError::log(store, log).into(),
        }
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
