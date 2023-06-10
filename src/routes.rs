use chrono::{Local, Utc};
use sailfish::TemplateOnce;
use serde::{Deserialize, Serialize};
use worker::{Cache, Headers, Request, Response, RouteContext};

#[derive(Deserialize)]
struct CreatePost {
    title: String,
    content: String,
    secret: String,
}

#[derive(Serialize, Deserialize)]
struct CreateResult {
    id: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct Post {
    id: u64,
    title: String,
    content: String,
    create_time: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Index {
    posts: Vec<Post>,
}

#[derive(TemplateOnce)]
#[template(path = "create.html")]
struct CreateTemplate {
    ctx: TempContext,
}

#[derive(TemplateOnce)]
#[template(path = "post.html")]
struct PostTemplate {
    data: Post,
    ctx: TempContext,
}

#[derive(TemplateOnce)]
#[template(path = "index.html")]
struct IndexTemplate {
    data: Index,
    ctx: TempContext,
}

#[derive(Debug, Serialize)]
struct TempContext {
    blog_name: String,
    icon: Option<String>,
}

impl TempContext {
    fn from_ctx<C>(ctx: &RouteContext<C>) -> Self {
        TempContext {
            blog_name: ctx
                .var("BLOG_NAME")
                .map(|var| var.to_string())
                .unwrap_or_else(|_| "unknown".to_string()),
            icon: ctx
                .var("BLOG_ICON")
                .map(|var| var.to_string())
                .ok(),
        }
    }
}

impl Post {
    fn with_local_time(&mut self) {
        self.create_time = chrono::DateTime::parse_from_rfc3339(&self.create_time)
            .unwrap()
            .with_timezone(&Local)
            .to_rfc3339();
    }
}

pub async fn create<C>(mut req: Request, ctx: RouteContext<C>) -> worker::Result<Response> {
    let post = req.json::<CreatePost>().await?;
    if ctx.secret("BLOG_PWD").ok().filter(|secret| &secret.to_string() == &post.secret).is_none() {
        return Response::error("no authentication", 403);
    };
    let db = ctx.env.d1("posts")?;
    let id = db
        .prepare("INSERT INTO posts (title, content, create_time) VALUES (?, ?, ?) returning (id);")
        .bind(&[
            post.title.into(),
            post.content.into(),
            Utc::now().to_rfc3339().into(),
        ])?
        .first::<CreateResult>(None)
        .await?
        .ok_or_else(|| worker::Error::RustError("insert post but no id is returned".to_owned()))?;

    if let Ok(url) = req.url() {
        let cache = Cache::default();
        cache
            .delete(
                {
                    let mut key = url.clone();
                    key.set_path("/");
                    key
                }
                    .as_str(),
                true,
            )
            .await?;
        cache
            .delete(
                {
                    let mut key = url.clone();
                    key.set_path("/json");
                    key
                }
                    .as_str(),
                true,
            )
            .await?;
    }
    /*let id = uuid::Uuid::new_v4().to_string();
    let kv = ctx.kv("worker-blog")?;
    kv.put(
        &id,
        Post {
            title: post.title,
            content: post.content,
            create_time: Utc::now().to_rfc3339(),
        },
    )?
    .execute()
    .await?;*/

    Response::from_json(&id)
}

pub async fn create_page<C>(_: Request, ctx: RouteContext<C>) -> worker::Result<Response> {
    let temp = CreateTemplate {
        ctx: TempContext::from_ctx(&ctx)
    };
    Response::from_html(
        temp.render_once()
            .map_err(|e| worker::Error::RustError(e.to_string()))?,
    )
}

pub async fn get_post<C>(req: Request, ctx: RouteContext<C>) -> worker::Result<Response> {
    let json = req.path().ends_with("/json");
    let cache = Cache::default();
    if let Some(resp) = cache.get(&req, false).await? {
        return Ok(resp);
    }

    if let Some(id) = ctx.param("id") {
        let db = ctx.env.d1("posts")?;
        let post = db
            .prepare("select * from posts where id = ?;")
            .bind(&[id.into()])?
            .first::<Post>(None)
            .await?;

        /* let kv = ctx.kv("worker-blog")?;
        if let Some(mut post) = kv.get(id).json::<Post>().await? {*/
        if let Some(mut post) = post {
            let mut resp = if json {
                Response::from_json(&post)?
            } else {
                post.with_local_time();
                let temp = PostTemplate {
                    data: post,
                    ctx: TempContext::from_ctx(&ctx),
                };
                Response::from_html(
                    temp.render_once()
                        .map_err(|e| worker::Error::RustError(e.to_string()))?,
                )?
            };
            cache
                .put(
                    &req,
                    resp.cloned()?.with_headers(Headers::from_iter(&[
                        // 24h
                        ("Cache-Control", "s-maxage=86400"),
                    ])),
                )
                .await?;
            Ok(resp)
        } else {
            Response::error("Post not found", 404)
        }
    } else {
        Err(worker::Error::RustError("missing the id param".to_owned()))
    }
}

pub async fn get_index<C>(req: Request, ctx: RouteContext<C>) -> worker::Result<Response> {
    let json = req.path().ends_with("/json");
    let cache = Cache::default();

    if let Some(resp) = cache.get(&req, true).await? {
        return Ok(resp);
    }

    /*let kv = ctx.kv("worker-blog")?;
    let list = kv.list().execute().await?;
    let mut posts = Vec::with_capacity(list.keys.len());
    for key in list.keys {
        let post = kv.get(&key.name).json::<Post>().await?.unwrap();
        posts.push(post);
    };*/
    let db = ctx.env.d1("posts")?;
    let mut posts = db
        .prepare("select * from posts;")
        .all()
        .await?
        .results::<Post>()?;
    posts.reverse();
    posts.iter_mut().for_each(|post| post.with_local_time());

    let index = Index { posts };
    let mut resp = if json {
        Response::from_json(&index)?
    } else {
        let temp = IndexTemplate {
            data: index,
            ctx: TempContext::from_ctx(&ctx),
        };
        Response::from_html(
            temp.render_once()
                .map_err(|e| worker::Error::RustError(e.to_string()))?,
        )?
    };
    cache.put(&req, resp.cloned()?.with_headers(Headers::from_iter(&[
        // 24h
        ("Cache-Control", "s-maxage=86400"),
    ]))).await?;
    Ok(resp)
}
