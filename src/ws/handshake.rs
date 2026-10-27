use crate::{HeaderMap, Response, StatusCode, ws_accept_key};

#[must_use]
pub fn accept_key(client_key: &str) -> String {
    ws_accept_key(client_key)
}

#[must_use]
pub fn is_upgrade_request(headers: &HeaderMap) -> bool {
    let has_upgrade = headers
        .get("upgrade")
        .is_some_and(|u| u.eq_ignore_ascii_case("websocket"));
    let has_conn = headers
        .get("connection")
        .is_some_and(|c| c.to_ascii_lowercase().contains("upgrade"));
    let has_key = headers.get("sec-websocket-key").is_some();

    has_upgrade && has_conn && has_key
}

#[must_use]
pub fn upgrade_response(client_key: &str) -> Response {
    let accept = accept_key(client_key);
    let mut res = Response::new(StatusCode::SwitchingProtocols);
    res.header("upgrade", "websocket");
    res.header("connection", "Upgrade");
    res.header("sec-websocket-accept", &accept);
    res
}
