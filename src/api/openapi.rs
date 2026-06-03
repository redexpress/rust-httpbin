use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Axum Httpbin",
        version = "0.2.0",
        description = "A lightweight HTTP testing service inspired by httpbin.org"
    ),
    paths(
        crate::endpoints::request::get::handler,
        crate::endpoints::request::post::handler,
        crate::endpoints::request::put::handler,
        crate::endpoints::request::patch::handler,
        crate::endpoints::request::delete::handler,
        crate::endpoints::inspect::headers::handler,
        crate::endpoints::inspect::ip::handler,
        crate::endpoints::inspect::user_agent::handler,
        crate::endpoints::inspect::cookies::get_cookies,
        crate::endpoints::inspect::cookies::set_cookies,
        crate::endpoints::inspect::cookies::set_cookie_one,
        crate::endpoints::inspect::cookies::delete_cookies,
        crate::endpoints::response::status::handler,
        crate::endpoints::response::delay::handler,
        crate::endpoints::response::redirect::handler,
        crate::endpoints::response::stream::handler,
        crate::endpoints::response::response_headers::handler,
        crate::endpoints::auth::basic::handler,
        crate::endpoints::auth::bearer::handler,
        crate::endpoints::utility::uuid::handler,
        crate::endpoints::utility::image::serve_png,
        crate::endpoints::utility::image::serve_jpeg,
        crate::endpoints::utility::image::serve_webp,
        crate::endpoints::utility::image::serve_svg,
    ),
    tags(
        (name = "request", description = "Echo request methods"),
        (name = "inspect", description = "Header / IP / User-Agent / Cookie inspection"),
        (name = "response", description = "Status / Delay / Redirect / Stream / response-headers control"),
        (name = "auth", description = "HTTP authentication endpoints"),
        (name = "utility", description = "UUID, catch-all, image fixtures")
    )
)]
pub struct ApiDoc;
