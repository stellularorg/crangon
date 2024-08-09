# 🦐 Crangon

Crangon-compatible pastebin using [`pastemd`](https://github.com/hkauso/pastemd).

## Configuration

Crangon supports the following configuration options through environment variables:

* `SITE_NAME` - the name of the site
* `INFO_URL` - the url (relative to root `/`) that will be served from the "what" link in the footer
  * Link is not shown in the footer if this variable is not set
* `GUPPY_ROOT` - the root url of a [Guppy](https://github.com/stellularorg/guppy) server
  * User authentication is completely disabled if this is not provided
  * When provided, views switch from [`OpenMultiple`](https://docs.rs/pastemd/latest/pastemd/database/enum.ViewMode.html#variant.OpenMultiple), to [`AuthenticatedOnce`](https://docs.rs/pastemd/latest/pastemd/database/enum.ViewMode.html#variant.AuthenticatedOnce)

The following configuration options are required for all database types (besides sqlite):

* `DB_TYPE` - the type of the database (`mysql` or `postgres`)
* `DB_HOST` - the database host location (likely `localhost`)
* `DB_USER` - database username
* `DB_PASS` - database password
* `DB_NAME` - database name
