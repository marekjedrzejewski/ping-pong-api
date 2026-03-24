use axum::{Router, response::Html, routing::get};

pub fn create_api_docs() -> Router {
    Router::new()
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
