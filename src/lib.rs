mod routes;

use worker::{console_log, event, Env, Request, Response, Result, Router};

#[inline(always)]
#[event(fetch)]
pub async fn main(req: Request, env: Env, ctx: worker::Context) -> Result<Response> {
    console_error_panic_hook::set_once();
    _main(req, env, ctx).await
}

pub async fn _main(req: Request, env: Env, _: worker::Context) -> Result<Response> {
    console_log!(
        "{} {}, located at: {:?}, within: {}",
        req.method().to_string(),
        req.path(),
        req.cf().coordinates().unwrap_or_default(),
        req.cf().region().unwrap_or("unknown region".into())
    );

    let router = Router::new();

    router
        .get_async("/sitemap.xml", routes::sitemap)
        .get_async("/create", routes::create_page)
        .post_async("/create", routes::create)
        .get_async("/json", routes::get_index)
        .get_async("/", routes::get_index)
        .get_async("/:id/json", routes::get_post)
        .get_async("/:id/", routes::get_post)
        .get_async("/:id", routes::get_post)
        .run(req, env)
        .await
}
