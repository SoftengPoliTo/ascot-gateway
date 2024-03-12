use rocket::http::uri::Origin;
use rocket::http::Status;
use rocket::Request;

use rocket_dyn_templates::{context, Template};

// Go to devices message.
const GO_TO_DEVICES_MESSAGE: &str = "Go to devices";
// Unknown error.
const UNKNOWN_ERROR_MESSAGE: &str = "Unknown";

struct RenderTemplate;

impl RenderTemplate {
    fn text(uri: &Origin<'_>, status: u16, error_message: &str) -> Template {
        Self::render(uri, "/", status, error_message)
    }

    fn render(uri: &Origin<'_>, route: &str, status: u16, error_message: &str) -> Template {
        Template::render(
            "error",
            context! {
                route,
                uri,
                status,
                error_message,
                goto_message: GO_TO_DEVICES_MESSAGE,
            },
        )
    }
}

#[derive(Responder)]
#[response(status = 500, content_type = "html")]
pub(crate) struct InternalError(Template);

impl InternalError {
    // Render a text containing an internal error
    pub(crate) fn text(uri: &Origin<'_>, error_message: &str) -> Self {
        Self(RenderTemplate::text(uri, 500, error_message))
    }
}

#[inline(always)]
pub(crate) async fn query_error<T, K: ToString>(
    function: impl std::future::Future<Output = Result<T, K>>,
    uri: &Origin<'_>,
) -> Result<T, InternalError> {
    function
        .await
        .map_err(|e| InternalError::text(uri, &e.to_string()))
}

// Renders the template for any other kind of catchers
#[catch(default)]
pub(crate) fn default(status: Status, req: &Request<'_>) -> Template {
    RenderTemplate::text(
        req.uri(),
        status.code,
        status.reason().unwrap_or(UNKNOWN_ERROR_MESSAGE),
    )
}

// Returns all defined catchers
pub(crate) fn catchers() -> Vec<rocket::Catcher> {
    catchers![default]
}
