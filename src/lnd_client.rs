use crate::common::*;
use futures::{future::FutureResult, Future, Stream};
use grpc::{ClientStub, Metadata, RequestOptions};
use lnd_rust::{
    macaroon_data::MacaroonData,
    rpc::{
        AddInvoiceResponse, FeeLimit, FeeLimit_oneof_limit, InvoiceSubscription,
        Invoice_InvoiceState, SendRequest, SendResponse,
    },
    rpc_grpc::{Lightning, LightningClient},
    tls_certificate::TLSCertificate,
};
use std::{
    default::Default,
    i64, io,
    net::{Ipv4Addr, SocketAddr},
    path::Path,
    sync::Arc,
    time::SystemTime,
};

const BACKEND_NAME: &str = "lnd";

impl LightningNode for (LightningClient, MacaroonData) {
    fn create_invoice(&self, satoshis: Satoshis) -> DynFut<Invoice, CreateInvoiceError> {
        let (client, macaroon) = self;
        let num_satoshis: i64 = match satoshis.checked_to_i64() {
            Some(sat) => sat,
            None => {
                return Box::new(FutureResult::from(Err(CreateInvoiceError::Unknown(
                    format!("invoice amount {} overflowed max value for lnd", satoshis.0),
                ))));
            }
        };
        let invoice = create_lnd_invoice(num_satoshis);
        let metadata: Metadata = macaroon.metadata();
        let requestoptions = RequestOptions { metadata };
        let response = client
            .add_invoice(requestoptions, invoice)
            .drop_metadata()
            .map_err(|grpc_err| CreateInvoiceError::Network {
                backend_name: BACKEND_NAME.to_owned(),
                err: format!("{:?}", grpc_err),
            })
            .and_then(|response: AddInvoiceResponse| {
                parse_bolt11(&response.payment_request).map_err(CreateInvoiceError::InvalidInvoice)
            });
        Box::new(response)
    }

    fn pay_invoice(
        &self,
        invoice: Invoice,
        amount: Satoshis,
        max_fee: Fee<Satoshis>,
    ) -> DynFut<PaidInvoiceOutgoing, PayError> {
        let (client, macaroon) = self;
        let iamount = match amount.checked_to_i64() {
            Some(i) => i,
            None => {
                return Box::new(FutureResult::from(Err(PayError::Unknown(format!(
                    "payment amount {} overflowed max value for lnd",
                    amount.0
                )))));
            }
        };
        let imax_fee = match max_fee.0.checked_to_i64() {
            Some(i) => i,
            None => {
                return Box::new(FutureResult::from(Err(PayError::Unknown(format!(
                    "payment max_fee {} overflowed max value for lnd",
                    (max_fee.0).0
                )))));
            }
        };
        let request = create_lnd_send_request(&invoice, iamount, imax_fee);
        let fut = client
            .send_payment_sync(
                RequestOptions {
                    metadata: macaroon.metadata(),
                },
                request,
            )
            .drop_metadata()
            .map_err(|err| PayError::Unknown(format!("{:?}", err)))
            .and_then(
                move |SendResponse {
                          payment_error,
                          payment_preimage,
                          payment_hash: phash,
                          payment_route,
                          ..
                      }| {
                    // lnd api documentation is unclear on how errors are reported here.
                    // we assume an empty preimage, or a preimage not matching the original hash
                    // indicates and error
                    let preimage = U256::try_from_slice(&payment_preimage)
                        .ok_or(PayError::Unknown(payment_error))?;
                    debug_assert_eq!(payment_preimage.len(), 32);
                    debug_assert_eq!(phash.len(), 32);
                    debug_assert_eq!(
                        U256::try_from_slice(&phash).unwrap(),
                        get_payment_hash(&invoice)
                    );
                    let ifees: i64 = payment_route
                        .into_option()
                        .ok_or_else(|| {
                            PayError::Unknown(format!("lnd did not report payment route"))
                        })?
                        .total_fees;
                    let fees_paid = Fee(Satoshis(to_unsigned(ifees).ok_or_else(|| {
                        PayError::Unknown(format!("lnd reported negative fees {}", ifees))
                    })?));
                    let paid_invoice = PaidInvoice::create(invoice, Preimage(preimage), amount)?;
                    let paid_invoice_outgoing = PaidInvoiceOutgoing {
                        paid_invoice,
                        fees_offered: max_fee,
                        fees_paid,
                    };
                    Ok(paid_invoice_outgoing)
                },
            );
        Box::new(fut)
    }

    fn paid_invoices(
        &self,
    ) -> crate::lighting_node::DynStream<PaidInvoice, SubscribePaidInvoicesError> {
        let (client, macaroon) = self;
        let sub = InvoiceSubscription {
            settle_index: 1,
            ..Default::default()
        };
        let stream = client
            .subscribe_invoices(
                RequestOptions {
                    metadata: macaroon.metadata(),
                },
                sub,
            )
            .drop_metadata()
            .map_err(|err| SubscribePaidInvoicesError::Unknown(format!("{:?}", err)))
            .and_then(|lnd_iv| {
                to_paid_invoice(lnd_iv)
                    .map_err(|err| SubscribePaidInvoicesError::Unknown(format!("{:?}", err)))
            });
        Box::new(stream)
    }
}

// Error initializing an LndClient
#[derive(Debug)]
pub enum CreateError {
    Io(io::Error),
    Tls(tls_api::Error),
    Grpc(grpc::Error),
}

impl From<io::Error> for CreateError {
    fn from(other: io::Error) -> Self {
        CreateError::Io(other)
    }
}

impl From<tls_api::Error> for CreateError {
    fn from(other: tls_api::Error) -> Self {
        CreateError::Tls(other)
    }
}

impl From<grpc::Error> for CreateError {
    fn from(other: grpc::Error) -> Self {
        CreateError::Grpc(other)
    }
}

pub fn init_default_lightning_client() -> Result<(LightningClient, MacaroonData), CreateError> {
    // TODO, don't hardcode.
    init_lightning_client(
        Path::new("/Volumes/btcchain/persist/lnd/tls.cert"),
        Path::new("/Volumes/btcchain/persist/lnd/data/chain/bitcoin/mainnet/admin.macaroon"),
        SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 10009),
    )
}

fn init_lightning_client(
    tls_cert: &Path,
    macaroon: &Path,
    addr: SocketAddr,
) -> Result<(LightningClient, MacaroonData), CreateError> {
    let certificate = TLSCertificate::from_path(tls_cert)?;
    let macaroon = MacaroonData::from_file_path(macaroon)?;
    let config = Default::default();
    let tls = certificate.into_tls("localhost")?;
    let grpc_client = grpc::Client::new_expl(&addr, "localhost", tls, config)?;
    Ok((
        LightningClient::with_client(Arc::new(grpc_client)),
        macaroon,
    ))
}

fn create_lnd_invoice(num_satoshis: i64) -> lnd_rust::rpc::Invoice {
    let random_preimage = U256::random();
    let hash_of_preimage = random_preimage.hash();
    let current_time: i64 = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("System time thinks we are living before the unix epoch.")
        .as_secs() as i64;
    lnd_rust::rpc::Invoice {
        memo: "".to_owned(),
        receipt: vec![],
        r_preimage: random_preimage.to_vec(),
        r_hash: hash_of_preimage.to_vec(),
        value: num_satoshis,
        settled: false,
        creation_date: current_time,
        settle_date: i64::max_value(), // not settled
        // We expect lnd to populate this field for us and send it as a response.
        // Todo, debug_assert that they did return a valid response.
        payment_request: "".to_owned(),
        // none for now, later we may use this field to uniquely track invoices
        description_hash: vec![],
        expiry: 3600,                    // one hour from now
        fallback_addr: "".to_owned(),    // none for now
        cltv_expiry: 9,                  // 9 is the default
        route_hints: Default::default(), // RepeatedField<RouteHint>,
        private: false,                  // we aren't using private channels, so it doesn't matter
        // We expect lnd to populate this for us.
        // TODO debug_assert this value is changed on return
        add_index: u64::min_value(),
        settle_index: u64::min_value(),  // not settled
        amt_paid: i64::min_value(),      // this field is depricated
        amt_paid_sat: i64::min_value(),  // not settled
        amt_paid_msat: i64::min_value(), // not settled
        state: Invoice_InvoiceState::OPEN,
        unknown_fields: Default::default(),
        cached_size: Default::default(),
    }
}

fn create_lnd_send_request(
    invoice: &Invoice,
    amount_satoshis: i64,
    fee_limit_satoshis: i64,
) -> SendRequest {
    let payment_request: String = to_bolt11(&invoice);
    let fee_limit = Some(FeeLimit {
        limit: Some(FeeLimit_oneof_limit::fixed(fee_limit_satoshis)),
        ..Default::default()
    })
    .into();
    SendRequest {
        dest: Default::default(),
        dest_string: Default::default(), // Lnd needs to infer this from the payment_request,
        amt: amount_satoshis,
        payment_hash: Default::default(),
        payment_hash_string: Default::default(), // This field expects a hex_encoded version.
        payment_request,
        final_cltv_delta: Default::default(), // TODO, figure out what this means, verify using a default value is appropriate.
        fee_limit: fee_limit,
        unknown_fields: Default::default(),
        cached_size: Default::default(),
    }
}

fn to_unsigned(a: i64) -> Option<u64> {
    if a < 0 {
        None
    } else {
        Some(a as u64)
    }
}

fn to_paid_invoice(iv: lnd_rust::rpc::Invoice) -> Result<PaidInvoice, ToPaidInvoiceError> {
    let lnd_rust::rpc::Invoice {
        payment_request,
        r_preimage,
        amt_paid_sat,
        state,
        ..
    } = iv;
    let invoice =
        parse_bolt11(&payment_request).map_err(ToPaidInvoiceError::InvalidPaymentRequest)?;
    if state == lnd_rust::rpc::Invoice_InvoiceState::OPEN {
        return Err(ToPaidInvoiceError::NotSettled);
    }
    let amount = Satoshis(
        to_unsigned(amt_paid_sat).ok_or(ToPaidInvoiceError::NegativeAmount(amt_paid_sat))?,
    );
    let preimage =
        Preimage(U256::try_from_slice(&r_preimage).ok_or(ToPaidInvoiceError::PreimageNotProvided)?);
    PaidInvoice::create(invoice, preimage, amount).map_err(ToPaidInvoiceError::PaidInvoiceInvalid)
}

#[derive(Debug, Clone)]
pub enum ToPaidInvoiceError {
    InvalidPaymentRequest(lightning_invoice::ParseOrSemanticError),
    NegativeAmount(i64),
    NotSettled,
    PreimageNotProvided,
    PaidInvoiceInvalid(PaidInvoiceInvalid),
}

#[cfg(test)]
mod test {
    use super::*;
    use futures::Future;
    use grpc::{Metadata, RequestOptions};
    use lnd_rust::rpc::GetInfoRequest;
    use lnd_rust::rpc_grpc::Lightning;

    #[test]
    fn info() {
        let (client, macaroon) = init_default_lightning_client().unwrap();
        let metadata: Metadata = macaroon.metadata();
        let requestoptions = RequestOptions { metadata };
        let getinforequest = GetInfoRequest::new();
        let fut_response = client.get_info(requestoptions, getinforequest);
        let (_metadata_pre, _response, _metadata_post) = fut_response.wait().unwrap();
    }

    #[test]
    fn create_invoice() {
        let node = init_default_lightning_client().unwrap();
        node.create_invoice(Satoshis(10)).wait().unwrap();
    }

    #[test]
    fn pay_invoice() {
        let node = init_default_lightning_client().unwrap();
        node.create_invoice(Satoshis(10)).wait().unwrap();
    }

    #[test]
    fn decode_payment_req() {
        // create a unique invoice
        // serialize invoice into bolt11
        // call rpc::DecodePayReq
        // verify returned fields are as expected
        // repeat for several invoice variants, attempt to test edge cases
        unimplemented!()
    }
}
