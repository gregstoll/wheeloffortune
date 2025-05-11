use search_corpus::process_query_string;

extern crate cgi;
extern crate json;

fn json_response_cross_origin(body: Vec<u8>) -> cgi::Response {
    let mut response = cgi::http::response::Builder::new().status(200).header(
        cgi::http::header::CONTENT_LENGTH,
        format!("{}", body.len()).as_str(),
    );
    response = response.header(cgi::http::header::CONTENT_TYPE, "application/json");
    response = response.header(cgi::http::header::ACCESS_CONTROL_ALLOW_ORIGIN, "*");

    response.body(body).unwrap()
}

fn success(s: json::JsonValue) -> cgi::Response {
    json_response_cross_origin(s.dump().as_bytes().to_vec())
}

fn error(s: &str) -> cgi::Response {
    json_response_cross_origin(
        json::object! {"error": s}
            .dump()
            .as_bytes()
            .to_vec(),
    )
}

fn process_request(request: &cgi::Request) -> Result<json::JsonValue, String> {
    let query = request
        .uri()
        .query()
        .ok_or(String::from("Internal error - no query string?"))?;
    process_query_string(query)
}

cgi::cgi_main! { |request: cgi::Request| {
    let result = process_request(&request);
    match result {
        Ok(val) => success(val),
        Err(err) => error(&err)
    }
} }
