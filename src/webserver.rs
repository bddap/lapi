use crate::common::*;
use futures::{future::FutureResult, Future, Sink, Stream};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::sync::Arc;
use warp::{
    filters::{
        body::{content_length_limit, json as filter_json},
        ws::{ws2, Message, WebSocket, Ws2},
    },
    get2, path, post2,
    reject::{self, Rejection},
    reply::Reply,
    Filter,
};

pub fn serve() -> Result<(), ServeError> {
    let api_low = ApiLow {
        database: FakeDb::new(),
        lighting_node: init_default_lightning_client().map_err(ServeError::Create)?,
    };
    let api_high = ApiHigh {
        api_low,
        log: FakeLog,
    };
    let s = server(api_high);
    warp::serve(s).run(([127, 0, 0, 1], 3030));
    Ok(())
}

pub fn server<D: Db, L: LightningNode, G: Log>(
    api_high: ApiHigh<D, L, G>,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> {
    let api = Arc::new(api_high);

    let post_json = post2().and(content_length_limit(1024 * 32));

    let post_invoice = path("invoice").and(filter_json()).and_then({
        let api = api.clone();
        move |req| api.generate_invoice(req).then(to_warp_result)
    });

    let post_pay = path("pay").and(filter_json()).and_then({
        let api = api.clone();
        move |req| api.pay_invoice(req).then(to_warp_result)
    });

    let get_balance = path!("balance" / Middle).and_then({
        let api = api.clone();
        move |middle| api.check_balance(middle).then(to_warp_result)
    });

    let get_invoice = path!("invoice" / PaymentHash).and_then({
        let api = api.clone();
        move |parm| api.check_invoice_status(parm).then(to_warp_result)
    });

    let await_invoice = path!("invoice" / PaymentHash).and(ws2()).and_then({
        let api = api.clone();
        move |ph: PaymentHash, conn: Ws2| {
            api.await_invoice_status(ph)
                .map_err(|_: ErrLogged| warp::reject::custom("server error"))
                .and_then(|response| to_websocket_message(response))
                .map(|message| {
                    conn.on_upgrade(|ws| {
                        ws.send(message)
                            .map(|_: WebSocket| ())
                            .map_err(|_: warp::Error| ())
                    })
                })
        }
    });

    post_json
        .and(post_invoice.or(post_pay))
        .or(get2().and(get_balance.or(await_invoice).or(get_invoice)))
}

fn to_warp_result<T: Serialize>(r: Result<T, ErrLogged>) -> Result<impl Reply, Rejection> {
    match r {
        Ok(t) => Ok(warp::reply::json(&t)),
        Err(logged) => Err(reject::custom("server error")),
    }
}

fn to_websocket_message<T: Serialize>(t: T) -> Result<Message, Rejection> {
    let ser = serde_json::to_string(&t).map_err(|_| reject::custom("response serialization error"));
    debug_assert!(ser.is_ok());
    ser.map(Message::text)
}

#[derive(Debug)]
pub enum ServeError {
    Create(CreateError),
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::api_types::*;
    use crate::test_util::*;

    macro_rules! server {
        () => (impl Filter<Extract = (impl Reply,), Error = Rejection> + 'static)
    }

    fn make_server_with_db<D: 'static + Db>(database: D) -> server!() {
        let api_low = ApiLow {
            database,
            lighting_node: init_default_lightning_client().unwrap(),
        };
        let api_high = ApiHigh {
            api_low,
            log: FakeLog,
        };
        server(api_high)
    }

    fn make_server() -> server!() {
        make_server_with_db(FakeDb::new())
    }

    fn js<T: Serialize>(t: T) -> String {
        serde_json::to_string(&t).unwrap()
    }

    fn sj<'a, T: Deserialize<'a>>(inp: &'a str) -> Result<T, String> {
        serde_json::from_str(inp).map_err(|_| inp.to_owned())
    }

    fn post<B: Serialize, R: DeserializeOwned>(server: &server!(), path: &str, body: B) -> R {
        let bod = js(body);
        let len = bod.as_bytes().len();
        let raw = warp::test::request()
            .path(path)
            .method("POST")
            .header("content-length", len)
            .body(&bod)
            .reply(server);
        sj(std::str::from_utf8(raw.body()).unwrap()).unwrap()
    }

    fn get<R: DeserializeOwned>(server: &server!(), path: &str) -> R {
        let raw = warp::test::request()
            .path(path)
            .method("POST")
            .reply(server);
        sj(std::str::from_utf8(raw.body()).unwrap()).unwrap()
    }

    fn new_invoice(server: &server!(), amount: u8, lesser: Lesser) -> GenerateInvoiceOk {
        let request = GenerateInvoiceRequest {
            lesser,
            satoshis: Satoshis(amount as u64),
        };
        let resp: GenerateInvoiceResponse = post(server, "/invoice", request);
        Into::<Result<_, _>>::into(resp).unwrap()
    }

    fn check_balance(server: &server!(), middle: Middle) -> CheckBalanceResponse {
        unimplemented!()
    }

    fn pay(
        server: &server!(),
        invoice: &Invoice,
        amount: Satoshis,
        master: Master,
    ) -> Result<PayInvoiceOk, PayInvoiceErr> {
        let resp: PayInvoiceResponse = post(
            server,
            "/pay",
            PayInvoiceRequest {
                master,
                invoice: InvoiceSerDe(invoice.clone()),
                amount_satoshis: amount,
                fee_satoshis: DEFAULT_FEE,
            },
        );
        Into::<Result<_, _>>::into(resp)
    }

    #[test]
    fn dev_workflow() {
        let server = make_server_with_db(db_with_account_a_balance());

        // generate account B
        let b = Master::random();

        // generate invoice for 1 sat using account B
        let invoice = new_invoice(&server, 1, b.into()).invoice.0;

        // pay invoice from account A
        pay(&server, &invoice, Satoshis(1), ACCOUNT_A).unwrap();

        // check B balance is 1 sat
        let bal: Result<_, _> = check_balance(&server, b.into()).into();
        assert_eq!(
            bal,
            Ok(CheckBalanceOk {
                balance_satoshis: Satoshis(1)
            })
        );
    }

    #[test]
    fn await_invoice() {
        use std::thread;

        let accnt_b = Master::random();
        let server = make_server_with_db(db_with_account_a_balance());
        let invoice = new_invoice(&server, 1, accnt_b.into()).invoice.0;
        let accnt_a_lesser: Lesser = ACCOUNT_A.into();

        // due to a limitation in warp::test, this line must come before wait_for_pay
        // TODO test wait_for_pay, pay in the correct order
        pay(&server, &invoice, Satoshis(2), ACCOUNT_A).unwrap();

        let mut websocket = warp::test::ws()
            .path(&format!("/invoice/{}", accnt_a_lesser))
            .handshake(server)
            .unwrap();
        let wait_for_pay = thread::spawn(move || {
            let resp_mesg = websocket.recv().unwrap();
            let resp_str = resp_mesg.to_str().unwrap();
            let resp: AwaitInvoiceResponse = sj(resp_str).unwrap();
            let res: Result<_, _> = resp.into();
            assert_eq!(
                res,
                Ok(AwaitInvoiceOk {
                    preimage: U256::zero(),
                    amount_paid_satoshis: Satoshis(2),
                })
            );
            websocket.recv_closed().unwrap(); // assert ws is closed afterward
        });

        wait_for_pay.join().unwrap(); // assert websocket listen thread succeeded
    }

    fn get_invoice_status(
        server: &server!(),
        payment_hash: PaymentHash,
    ) -> Result<CheckInvoiceOk, CheckInvoiceErr> {
        let path = format!("/invoice/{}", payment_hash);
        let res: CheckInvoiceResponse = get(server, &path);
        res.into()
    }

    fn get_balance(server: &server!(), middle: Middle) -> Result<CheckBalanceOk, CheckBalanceErr> {
        let path = format!("/balance/{}", middle);
        let res: CheckBalanceResponse = get(server, &path);
        res.into()
    }

    #[test]
    fn get_invoice() {
        let accnt_b = Master::random();
        let server = make_server_with_db(db_with_account_a_balance());
        let accnt_a_lesser: Lesser = ACCOUNT_A.into();

        // gobbleygook payment hash is an error
        assert_eq!(
            get_invoice_status(&server, PaymentHash::random()),
            Err(CheckInvoiceErr::NonExistent(()))
        );

        let invoice = new_invoice(&server, 1, accnt_b.into()).invoice.0;

        // Waiting
        assert_eq!(
            get_invoice_status(&server, get_payment_hash(&invoice)),
            Ok(CheckInvoiceOk::Waiting(()))
        );

        pay(&server, &invoice, Satoshis(2), ACCOUNT_A).unwrap();

        // After payment assert paid
        if let CheckInvoiceOk::Paid {
            preimage,
            amount_paid_satoshis,
        } = get_invoice_status(&server, get_payment_hash(&invoice)).unwrap()
        {
            assert_eq!(preimage.hash(), get_payment_hash(&invoice));
            assert_eq!(amount_paid_satoshis, Satoshis(2));
        } else {
            panic!()
        }

        assert_eq!(
            get_balance(&server, accnt_b.into()),
            Ok(CheckBalanceOk {
                balance_satoshis: Satoshis(2)
            })
        );
    }

    #[test]
    fn pay_to_self() {
        let server = make_server_with_db(db_with_account_a_balance());
        let balance_pre = get_balance(&server, ACCOUNT_A.into()).unwrap();
        let invoice = new_invoice(&server, 1, ACCOUNT_A.into()).invoice.0;
        pay(&server, &invoice, Satoshis(1), ACCOUNT_A).unwrap();
        let balance_post = get_balance(&server, ACCOUNT_A.into()).unwrap();

        // Assert payment hash match
        unimplemented!();
    }

    #[test]
    fn fail_with_no_balance() {
        let server = make_server();
        let account_b = Master::random();
        let invoice = new_invoice(&server, 0, account_b.into()).invoice.0;
        let res = pay(&server, &invoice, Satoshis(1), account_b);
        assert_eq!(res, Err(PayInvoiceErr::InsufficientBalance(())))
    }
}
