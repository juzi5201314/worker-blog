# Cloudflare Worker Blog

## Deploy

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

### Done
run `npm wrangler publish`