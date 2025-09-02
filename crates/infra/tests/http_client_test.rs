use nve_core::ports::http::HttpClient;
use nve_infra::http_client::ReqwestHttp;

use httpmock::prelude::*;
use serde::Deserialize;

#[tokio::test]
async fn get_bytes_ok_devuelve_el_payload_en_bytes() {
    let server = MockServer::start();

    let _m = server.mock(|when, then| {
        when.method(GET).path("/bytes");
        then.status(200).body("hola-bytes");
    });

    let url = format!("{}/bytes", server.base_url());
    let http = ReqwestHttp::default();

    let bytes = http.get_bytes(&url).await.expect("bytes ok");
    assert_eq!(bytes, b"hola-bytes");
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct Foo {
    name: String,
    id: u32,
}

#[tokio::test]
async fn get_json_ok_deserializa_el_json() {
    let server = MockServer::start();

    let _m = server.mock(|when, then| {
        when.method(GET).path("/json");
        then.status(200)
            .header("content-type", "application/json")
            .body(r#"{ "name": "Javi", "id": 42 }"#);
    });

    let url = format!("{}/json", server.base_url());
    let http = ReqwestHttp::default();

    let foo: Foo = http.get_json(&url).await.expect("json ok");
    assert_eq!(
        foo,
        Foo {
            name: "Javi".into(),
            id: 42
        }
    );
}

#[tokio::test]
async fn get_bytes_error_por_http_status_no_2xx() {
    let server = MockServer::start();

    let _m = server.mock(|when, then| {
        when.method(GET).path("/404");
        then.status(404).body("not found");
    });

    let url = format!("{}/404", server.base_url());
    let http = ReqwestHttp::default();

    let res = http.get_bytes(&url).await;
    assert!(res.is_err(), "esperábamos error por 404");
}

#[tokio::test]
async fn get_json_error_si_el_body_no_es_json_valido() {
    let server = MockServer::start();

    let _m = server.mock(|when, then| {
        when.method(GET).path("/bad-json");
        then.status(200)
            .header("content-type", "application/json")
            .body("esto no es json");
    });

    let url = format!("{}/bad-json", server.base_url());
    let http = ReqwestHttp::default();

    let res = http.get_json::<Foo>(&url).await;
    assert!(res.is_err(), "debe fallar al deserializar json inválido");
}

#[tokio::test]
async fn get_json_error_por_http_status_no_2xx() {
    let server = MockServer::start();

    let _m = server.mock(|when, then| {
        when.method(GET).path("/teapot");
        then.status(418).body(r#"{"message":"nope"}"#);
    });

    let url = format!("{}/teapot", server.base_url());
    let http = ReqwestHttp::default();

    let res = http.get_json::<Foo>(&url).await;
    assert!(res.is_err(), "esperábamos error por 418");
}
