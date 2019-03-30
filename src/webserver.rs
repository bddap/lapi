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

    const ACCOUNT_A: Master = Master(U256([
        0xda, 0xbd, 0xf8, 0xc5, 0x74, 0xfb, 0x9a, 0x9e, 0x27, 0x72, 0x05, 0xe2, 0xda, 0x3d, 0x38,
        0xf1, 0x49, 0x60, 0x8e, 0x34, 0x96, 0x8c, 0x1f, 0xf1, 0x5f, 0xb9, 0xf1, 0x83, 0xde, 0x5c,
        0x40, 0x00,
    ]));

    macro_rules! server {
        () => (impl Filter<Extract = (impl Reply,), Error = Rejection> + 'static)
    }

    fn make_server() -> server!() {
        let api_low = ApiLow {
            database: FakeDb::new(),
            lighting_node: init_default_lightning_client().unwrap(),
        };
        let api_high = ApiHigh {
            api_low,
            log: FakeLog,
        };
        server(api_high)
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

    fn new_invoice(server: &server!(), amount: u8, lesser: Lesser) -> GenerateInvoiceOk {
        let request = GenerateInvoiceRequest {
            lesser,
            satoshis: Satoshis(amount as u64),
        };
        let resp: GenerateInvoiceResponse = post(server, "/invoice", request);
        let res: Result<_, _> = resp.into();
        res.unwrap()
    }

    fn check_balance(server: &server!(), middle: Middle) -> CheckBalanceResponse {
        unimplemented!()
    }

    fn pay(server: &server!(), invoice: Invoice, master: Master) -> PayInvoiceResponse {
        unimplemented!()
    }

    #[test]
    fn dev_workflow() {
        let server = make_server();

        // generate account B
        let b = Master::random();

        // generate invoice for 1 sat using account B
        let invoice = new_invoice(&server, 1, b.into()).invoice.0;

        // pay invoice from account A
        pay(&server, invoice, ACCOUNT_A);

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
    fn client_workflow() {}

    #[test]
    fn await_invoice() {}
}
