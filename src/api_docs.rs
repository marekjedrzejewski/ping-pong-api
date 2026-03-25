use axum::{
    Router,
    response::{Html, Redirect},
    routing::get,
};

pub fn create_api_docs() -> Router {
    Router::new()
        .route("/", get(|| async { Redirect::permanent("/api-docs") }))
        .route("/api-docs", get(scalar_ui))
        .route("/api-docs/openapi.yaml", get(openapi_spec))
}

async fn openapi_spec() -> &'static str {
    include_str!("openapi.yaml")
}

async fn scalar_ui() -> Html<&'static str> {
    Html(
        r#"
<!doctype html>
<html>
  <head>
    <title>API Docs</title>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
  </head>
  <body>
    <script
      id="api-reference"
      data-url="/api-docs/openapi.yaml">
    </script>
    <script src="https://cdn.jsdelivr.net/npm/@scalar/api-reference"></script>
  </body>
</html>
    "#,
    )
}

#[cfg(test)]
mod tests {
    use super::create_api_docs;
    use axum::http::StatusCode;
    use axum_test::TestServer;

    const API_DOCS_PATH: &str = "/api-docs";
    fn setup_server() -> TestServer {
        TestServer::builder().build(create_api_docs()).unwrap()
    }

    #[tokio::test]
    async fn api_docs_served_from_root() {
        let server = setup_server();
        let root_response = server.get("/").await;
        root_response.assert_status(StatusCode::PERMANENT_REDIRECT);
        root_response.assert_header("location", API_DOCS_PATH);

        let docs_response = server.get(API_DOCS_PATH).await;
        docs_response.assert_status_ok();
        docs_response.assert_text_contains("API Docs");
    }

    // can't put all in one test. Content js-generated - transparent to tests.
    #[tokio::test]
    async fn openapi_spec_served() {
        let server = setup_server();
        let response = server.get("/api-docs/openapi.yaml").await;
        response.assert_status_ok();
        response.assert_text_contains("title: Ping Pong API");
    }
}
