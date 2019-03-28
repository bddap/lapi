use crate::common::*;
use futures::{future::FutureResult, Future, Sink, Stream};
use serde::{Deserialize, Serialize};
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
    let api_owned = ApiHigh {
        api_low,
        log: FakeLog,
    };
    let api = Arc::new(api_owned);

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

    let handler = post_json
        .and(post_invoice.or(post_pay))
        .or(get2().and(get_balance.or(await_invoice).or(get_invoice)));

    warp::serve(handler).run(([127, 0, 0, 1], 3030));
    Ok(())
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
