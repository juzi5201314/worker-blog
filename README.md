# Cloudflare Worker Blog

Demo: [My blog](https://worker-blog.soeur.workers.dev/)

`Worker Blog` is a simple personal blog system based on `Cloudflare Workers`, it uses `Cloudflare Workers` to process requests and routing, and `Cloudflare D1` to store data.

## Todo
At present, you can only add post but not modify or delete it. To complete these operations, you must enter the d1 database management manual operation from cloudflare dash (

## Configuration
after deploying, go to cloudflare dash -> the worker you just deployed.

next, go to `Settings` -> `Variables` -> `Environment Variables`:

`BLOG_NAME`: The name you set for your blog, which will appear on the title of the default template (or for other templates).

`BLOG_ICON`: The favicon of your website can be the url pointing to the original image or even `data:image/png;base64,XXXX`. (But cloudflare only allows you to put 1kb text in the environment variable)

`BLOG_PWD`: secret, used to create post. Note: click `Encrypt`

## Front-end
`Worker Blog` has a simple, unreliable but fast built-in template. You can create your own static pages and access the worker blog through the api, which is Client Site Rendering (CSR).

Of course, you can also change the built-in template and recompile and upload wasm to change your front-end page. Even change the rust source code to use other templating engines. But SSR has an obvious problem: the cpu time allowed by cloudflare worker is limited.

## Deploy
First, clone this
```
git clone https://github.com/juzi5201314/worker-blog
cd worker-blog
```

then, you must install [wrangler](https://github.com/cloudflare/workers-sdk).

`npm install wrangler --save-dev` or `npm install`

create `wrangler.toml`:
```toml
# wrangler.toml

name = "<worker name>"
main = "build/worker/shim.mjs"
compatibility_date = "2023-06-08"

[vars]
BLOG_NAME = "A worker blog!"
BLOG_ICON = ""

[[d1_databases]]
binding = "posts"
database_name = "<DATABASE_NAME>"
database_id = "<ignore first>"
```

next, [create database](#create-database).

### Create database
1. run `npm wrangler d1 create <DATABASE_NAME>`

> DATABASE_NAME = change to your preferred name

The output looks like:
```
✅ Successfully created DB '<DATABASE_NAME>'

[[d1_databases]]
binding = "DB" # ignore it
database_name = "<DATABASE_NAME>"
database_id = "<unique id>"
```

copy `<unique id>` value to `database_id` in `wrangler.toml`
```
# wrangler.toml

[[d1_databases]]
... # Keep `binding = "posts"`
database_name = "<DATABASE_NAME>"
database_id = "<Here!>"
```
2. run `npm wrangler d1 execute <DATABASE_NAME> --file=./create.sql`

### Publish
run `npm wrangler publish`

## Build
If you change any source code (including default templates), then you need to recompile.

1. install [Rust](https://www.rust-lang.org/)
2. add wasm target: `rustup target add wasm32-unknown-unknown`
3. install worker-build: `cargo install worker-build`
4. run `worker-build --release`
5. [publish](#publish)

### Route

* `GET /create`: Use the built-in template to render the page that creates a new post

* `POST /create`:
#### Request:
```json5
{
  "title": "post title",
  "content": "post content",
  "secret": "<user-entered secret>"
}
```
#### Response
```json5
{
  "id": 0 //new post id
}
```

* `GET /`: The index page rendered using the built-in template

* `GET /json`:

see [`Post`](#response--post-)
#### Response
```
{
  "posts": [<Post>]
}
```

* `GET /:post_id`: The post page rendered using the built-in template

* `GET /:post_id/json`:
#### Response (Post)
```json5
{
  "id": 0,
  "title": "post title",
  "content": "post content",
  "create_time": "post release time in rfc3339 format"
}
```

* `GET /sitemap.xml`: ummm..