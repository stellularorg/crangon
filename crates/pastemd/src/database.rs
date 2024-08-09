use crate::model::{PasteCreate, PasteClone, PasteError, Paste, PasteMetadata};

use dorsal::utility;
use dorsal::query as sqlquery;
use dorsal::db::special::auth_db::{FullUser, UserMetadata};

pub type Result<T> = std::result::Result<T, PasteError>;

#[derive(Clone, Debug, PartialEq)]
pub enum ViewMode {
    /// Only authenticated users can count as a paste view and only once
    AuthenticatedOnce,
    /// Anybody can count as a paste view multiple times;
    /// views are only stored in redis when using this mode
    OpenMultiple,
}

#[derive(Clone, Debug)]
pub struct PastesTableConfig {
    /// The name of the table
    pub table_name: String,
    /// The caching prefix associated with the table
    pub prefix: String,
    // columns
    /// Mapping for the `id` column
    pub id: String,
    /// Mapping for the `url` column
    pub url: String,
    /// Mapping for the `password` column
    pub password: String,
    /// Mapping for the `content` column
    pub content: String,
    /// Mapping for the `date_published` column
    pub date_published: String,
    /// Mapping for the `date_edited` column
    pub date_edited: String,
    /// Mapping for the `metadata` column
    pub metadata: String,
}

impl Default for PastesTableConfig {
    fn default() -> Self {
        Self {
            table_name: "pastes".to_string(),
            prefix: "paste".to_string(),
            // columns
            id: "id".to_string(),
            url: "url".to_string(),
            password: "password".to_string(),
            content: "content".to_string(),
            date_published: "date_published".to_string(),
            date_edited: "date_edited".to_string(),
            metadata: "metadata".to_string(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ViewsTableConfig {
    /// The name of the table
    pub table_name: String,
    /// The caching prefix associated with the table
    pub prefix: String,
}

impl Default for ViewsTableConfig {
    fn default() -> Self {
        Self {
            table_name: "views".to_string(),
            prefix: "views".to_string(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ServerOptions {
    /// If pastes can require a password to be viewed
    pub view_password: bool,
    /// If authentication through guppy is enabled
    pub guppy: bool,
    /// If pastes can have a owner username (guppy required)
    pub paste_ownership: bool,
    /// View mode options
    pub view_mode: ViewMode,
    /// Pastes table config
    pub table_pastes: PastesTableConfig,
    /// Views table config
    pub table_views: ViewsTableConfig,
}

impl ServerOptions {
    /// Enable all options
    pub fn truthy() -> Self {
        Self {
            view_password: true,
            guppy: true,
            paste_ownership: true,
            view_mode: ViewMode::OpenMultiple,
            table_pastes: PastesTableConfig::default(),
            table_views: ViewsTableConfig::default(),
        }
    }
}

impl Default for ServerOptions {
    fn default() -> Self {
        Self {
            view_password: false,
            guppy: false,
            paste_ownership: false,
            view_mode: ViewMode::OpenMultiple,
            table_pastes: PastesTableConfig::default(),
            table_views: ViewsTableConfig::default(),
        }
    }
}

/// Database connector
#[derive(Clone)]
pub struct Database {
    pub base: dorsal::StarterDatabase,
    pub auth: dorsal::AuthDatabase,
    pub options: ServerOptions,
}

impl Database {
    pub async fn new(opts: dorsal::DatabaseOpts, opts1: ServerOptions) -> Self {
        let base = dorsal::StarterDatabase::new(opts).await;

        Self {
            base: base.clone(),
            auth: dorsal::AuthDatabase::new(
                base,
                dorsal::db::special::auth_db::DatabaseOptions::default(),
            )
            .await,
            options: opts1,
        }
    }

    /// Init database
    pub async fn init(&self) {
        // create tables
        let c = &self.base.db.client;

        let _ = sqlquery(&format!(
            "CREATE TABLE IF NOT EXISTS \"{}\" (
                 {} TEXT,
                 {} TEXT,
                 {} TEXT,
                 {} TEXT,
                 {} TEXT,
                 {} TEXT,
                 {} TEXT
            )",
            // table
            self.options.table_pastes.table_name,
            // columns
            self.options.table_pastes.url,
            self.options.table_pastes.id,
            self.options.table_pastes.password,
            self.options.table_pastes.date_published,
            self.options.table_pastes.date_edited,
            self.options.table_pastes.content,
            self.options.table_pastes.metadata
        ))
        .execute(c)
        .await;

        if self.options.view_mode == ViewMode::AuthenticatedOnce {
            // create table to track views
            let _ = sqlquery(&format!(
                "CREATE TABLE IF NOT EXISTS \"{}\" (
                    url      TEXT,
                    username TEXT
                )",
                self.options.table_views.table_name
            ))
            .execute(c)
            .await;
        }
    }

    // ...

    /// Get an existing paste by `url`
    ///
    /// # Arguments
    /// * `url` - [`String`] of the paste's `url` field
    pub async fn get_paste_by_url(&self, mut url: String) -> Result<Paste> {
        url = idna::punycode::encode_str(&url).unwrap().to_lowercase();

        if url.ends_with("-") {
            url.pop();
        }

        // check in cache
        match self
            .base
            .cachedb
            .get(format!("{}:{}", self.options.table_pastes.prefix, url))
            .await
        {
            Some(c) => return Ok(serde_json::from_str::<Paste>(c.as_str()).unwrap()),
            None => (),
        };

        // pull from database
        let query: String = if (self.base.db._type == "sqlite") | (self.base.db._type == "mysql") {
            "SELECT * FROM \":t\" WHERE \":url\" = ?"
        } else {
            "SELECT * FROM \":t\" WHERE \":url\" = $1"
        }
        .to_string()
        .replace(":t", &self.options.table_pastes.table_name)
        .replace(":url", &self.options.table_pastes.url);

        let c = &self.base.db.client;
        let res = match sqlquery(&query)
            .bind::<&String>(&url.to_lowercase())
            .fetch_one(c)
            .await
        {
            Ok(p) => self.base.textify_row(p).data,
            Err(_) => return Err(PasteError::NotFound),
        };

        // return
        let paste = Paste {
            id: res.get(&self.options.table_pastes.id).unwrap().to_string(),
            url: res.get(&self.options.table_pastes.url).unwrap().to_string(),
            password: res
                .get(&self.options.table_pastes.password)
                .unwrap()
                .to_string(),
            content: res
                .get(&self.options.table_pastes.content)
                .unwrap()
                .to_string(),
            date_published: res
                .get(&self.options.table_pastes.date_published)
                .unwrap()
                .parse::<u128>()
                .unwrap(),
            date_edited: res
                .get(&self.options.table_pastes.date_edited)
                .unwrap()
                .parse::<u128>()
                .unwrap(),
            metadata: match serde_json::from_str(
                res.get(&self.options.table_pastes.metadata).unwrap(),
            ) {
                Ok(m) => m,
                Err(e) => {
                    dbg!(e);
                    return Err(PasteError::ValueError);
                }
            },
        };

        // store in cache
        self.base
            .cachedb
            .set(
                format!("{}:{}", self.options.table_pastes.prefix, url),
                serde_json::to_string::<Paste>(&paste).unwrap(),
            )
            .await;

        // return
        Ok(paste)
    }

    /// Create a new paste
    ///
    /// # Arguments
    /// * `props` - [`PasteCreate`]
    ///
    /// # Returns
    /// * Result containing a tuple with the unhashed edit password and the paste
    pub async fn create_paste(&self, mut props: PasteCreate) -> Result<(String, Paste)> {
        props.url = idna::punycode::encode_str(&props.url)
            .unwrap()
            .to_lowercase();

        if props.url.ends_with("-") {
            props.url.pop();
        }

        // make sure paste doesn't already exist
        if let Ok(_) = self.get_paste_by_url(props.url.clone()).await {
            return Err(PasteError::AlreadyExists);
        }

        // create url if not supplied
        if props.url.is_empty() {
            props.url = utility::random_id().chars().take(10).collect();
        }

        // create random password if not supplied
        if props.password.is_empty() {
            props.password = utility::random_id().chars().take(10).collect();
        }

        // check lengths
        if (props.url.len() > 250) | (props.url.len() < 3) {
            return Err(PasteError::ValueError);
        }

        if (props.content.len() > 200_000) | (props.content.len() < 1) {
            return Err(PasteError::ValueError);
        }

        // (characters used)
        let regex = regex::RegexBuilder::new("^[\\w\\_\\-\\.\\!\\p{Extended_Pictographic}]+$")
            .multi_line(true)
            .build()
            .unwrap();

        if regex.captures(&props.url).iter().len() < 1 {
            return Err(PasteError::ValueError);
        }

        // ...
        let paste = Paste {
            id: utility::random_id(),
            url: props.url,
            content: props.content,
            password: utility::hash(props.password.clone()),
            date_published: utility::unix_epoch_timestamp(),
            date_edited: utility::unix_epoch_timestamp(),
            metadata: super::model::PasteMetadata::default(),
        };

        // create paste
        let query: String = if (self.base.db._type == "sqlite") | (self.base.db._type == "mysql") {
            "INSERT INTO \":t\" VALUES (?, ?, ?, ?, ?, ?, ?)"
        } else {
            "INSERT INTO \":t\" VALEUS ($1, $2, $3, $4, $5, $6, $7)"
        }
        .to_string()
        .replace(":t", &self.options.table_pastes.table_name);

        let c = &self.base.db.client;
        match sqlquery(&query)
            .bind::<&String>(&paste.url)
            .bind::<&String>(&paste.id)
            .bind::<&String>(&paste.password)
            .bind::<&String>(&paste.date_published.to_string())
            .bind::<&String>(&paste.date_edited.to_string())
            .bind::<&String>(&paste.content)
            .bind::<&String>(match serde_json::to_string(&paste.metadata) {
                Ok(ref s) => s,
                Err(_) => return Err(PasteError::ValueError),
            })
            .execute(c)
            .await
        {
            Ok(_) => return Ok((props.password, paste)),
            Err(e) => {
                dbg!(e);
                return Err(PasteError::Other);
            }
        };
    }

    /// Use an existing paste as a template
    ///
    /// # Arguments
    /// * `props` - [`PasteClone`]
    ///
    /// # Returns
    /// * Result containing a tuple with the unhashed edit password and the paste
    pub async fn clone_paste(&self, mut props: PasteClone) -> Result<(String, Paste)> {
        props.url = idna::punycode::encode_str(&props.url)
            .unwrap()
            .to_lowercase();

        if props.url.ends_with("-") {
            props.url.pop();
        }

        // make sure paste doesn't already exist
        if let Ok(_) = self.get_paste_by_url(props.url.clone()).await {
            return Err(PasteError::AlreadyExists);
        }

        // make sure paste source exists
        let source = match self.get_paste_by_url(props.source).await {
            Ok(p) => p,
            Err(e) => return Err(e),
        };

        // create url if not supplied
        if props.url.is_empty() {
            props.url = utility::random_id().chars().take(10).collect();
        }

        // create random password if not supplied
        if props.password.is_empty() {
            props.password = utility::random_id().chars().take(10).collect();
        }

        // check lengths
        if (props.url.len() > 250) | (props.url.len() < 3) {
            return Err(PasteError::ValueError);
        }

        // (characters used)
        let regex = regex::RegexBuilder::new("^[\\w\\_\\-\\.\\!\\p{Extended_Pictographic}]+$")
            .multi_line(true)
            .build()
            .unwrap();

        if regex.captures(&props.url).iter().len() < 1 {
            return Err(PasteError::ValueError);
        }

        // ...
        let source_c = source.clone();
        let paste = Paste {
            id: utility::random_id(),
            url: props.url,
            content: source.content,
            password: utility::hash(props.password.clone()),
            date_published: utility::unix_epoch_timestamp(),
            date_edited: utility::unix_epoch_timestamp(),
            metadata: super::model::PasteMetadata::from(source_c), // use other paste as a template
        };

        // create paste
        let query: String = if (self.base.db._type == "sqlite") | (self.base.db._type == "mysql") {
            "INSERT INTO \":t\" VALUES (?, ?, ?, ?, ?, ?, ?)"
        } else {
            "INSERT INTO \":t\" VALEUS ($1, $2, $3, $4, $5, $6, $7)"
        }
        .to_string()
        .replace(":t", &self.options.table_pastes.table_name);

        let c = &self.base.db.client;
        match sqlquery(&query)
            .bind::<&String>(&paste.url)
            .bind::<&String>(&paste.id)
            .bind::<&String>(&paste.password)
            .bind::<&String>(&paste.date_published.to_string())
            .bind::<&String>(&paste.date_edited.to_string())
            .bind::<&String>(&paste.content)
            .bind::<&String>(match serde_json::to_string(&paste.metadata) {
                Ok(ref s) => s,
                Err(_) => return Err(PasteError::ValueError),
            })
            .execute(c)
            .await
        {
            Ok(_) => return Ok((props.password, paste)),
            Err(e) => {
                dbg!(e);
                return Err(PasteError::Other);
            }
        };
    }

    /// Delete an existing paste by `url`
    ///
    /// # Arguments
    /// * `url` - the paste to delete
    /// * `password` - the paste's edit password
    pub async fn delete_paste_by_url(&self, mut url: String, password: String) -> Result<()> {
        url = idna::punycode::encode_str(&url).unwrap().to_lowercase();

        if url.ends_with("-") {
            url.pop();
        }

        // get paste
        let existing = match self.get_paste_by_url(url.clone()).await {
            Ok(p) => p,
            Err(err) => return Err(err),
        };

        // check password
        if utility::hash(password) != existing.password {
            return Err(PasteError::PasswordIncorrect);
        }

        // delete paste view count
        self.base
            .cachedb
            .remove(format!("{}:{}", self.options.table_views.prefix, url))
            .await;

        // delete paste
        let query: String = if (self.base.db._type == "sqlite") | (self.base.db._type == "mysql") {
            "DELETE FROM \":t\" WHERE \":url\" = ?"
        } else {
            "DELETE FROM \":t\" WHERE \":url\" = $1"
        }
        .to_string()
        .replace(":t", &self.options.table_pastes.table_name)
        .replace(":url", &self.options.table_pastes.url);

        let c = &self.base.db.client;
        match sqlquery(&query).bind::<&String>(&url).execute(c).await {
            Ok(_) => {
                // remove from cache
                self.base
                    .cachedb
                    .remove(format!("{}:{}", self.options.table_pastes.prefix, url))
                    .await;

                if self.options.view_mode == ViewMode::AuthenticatedOnce {
                    // delete all view logs
                    let query: String =
                        if (self.base.db._type == "sqlite") | (self.base.db._type == "mysql") {
                            "DELETE FROM \":t\" WHERE \"url\" = ?"
                        } else {
                            "DELETE FROM \":t\" WHERE \"url\" = $1"
                        }
                        .replace(":t", &self.options.table_views.table_name);

                    if let Err(_) = sqlquery(&query).bind::<&String>(&url).execute(c).await {
                        return Err(PasteError::Other);
                    };
                }

                // return
                return Ok(());
            }
            Err(_) => return Err(PasteError::Other),
        };
    }

    /// Edit an existing paste by `url`
    ///
    /// # Arguments
    /// * `url` - the paste to edit
    /// * `password` - the paste's edit password
    /// * `new_content` - the new content of the paste
    /// * `new_url` - the new url of the paste
    /// * `new_password` - the new password of the paste
    /// * `editing_as` - the userstate of the user we're editing the paste as
    pub async fn edit_paste_by_url(
        &self,
        mut url: String,
        password: String,
        new_content: String,
        mut new_url: String,
        mut new_password: String,
        editing_as: Option<FullUser<UserMetadata>>,
    ) -> Result<()> {
        url = idna::punycode::encode_str(&url).unwrap().to_lowercase();

        if url.ends_with("-") {
            url.pop();
        }

        // get paste
        let existing = match self.get_paste_by_url(url.clone()).await {
            Ok(p) => p,
            Err(err) => return Err(err),
        };

        // check password
        let mut skip_password_check: bool = false;

        if let Some(ua) = editing_as {
            // check if we're the paste owner
            if ua.user.username == existing.metadata.owner {
                skip_password_check = true;
            }
            // check if we have the "ManagePastes" permission
            else if ua.level.permissions.contains(&"ManagePastes".to_string()) {
                skip_password_check = true;
            }
        }

        if skip_password_check == false {
            if utility::hash(password) != existing.password {
                return Err(PasteError::PasswordIncorrect);
            }
        }

        // hash new password
        if !new_password.is_empty() {
            new_password = utility::hash(new_password);
        } else {
            new_password = existing.password;
        }

        // update new_url
        if new_url.is_empty() {
            new_url = existing.url;
        }

        new_url = idna::punycode::encode_str(&new_url).unwrap();

        if new_url.ends_with("-") {
            new_url.pop();
        }

        // edit paste
        let query: String = if (self.base.db._type == "sqlite") | (self.base.db._type == "mysql") {
            "UPDATE \":t\" SET \":content\" = ?, \":password\" = ?, \":url\" = ?, \":date_edited\" = ? WHERE \":url\" = ?"
        } else {
            "UPDATE \":t\" SET (\":content\" = $1, \":password\" = $2, \":url\" = $3, \":date_edited\" = $4) WHERE \":url\" = $5"
        }
        .to_string()
        .replace(":t", &self.options.table_pastes.table_name)
        .replace(":url", &self.options.table_pastes.url)
        .replace(":content", &self.options.table_pastes.content)
        .replace(":password", &self.options.table_pastes.password)
        .replace(":date_edited", &self.options.table_pastes.date_edited);

        let c = &self.base.db.client;
        match sqlquery(&query)
            .bind::<&String>(&new_content)
            .bind::<&String>(&new_password)
            .bind::<&String>(&new_url)
            .bind::<&String>(&utility::unix_epoch_timestamp().to_string())
            .bind::<&String>(&url)
            .execute(c)
            .await
        {
            Ok(_) => {
                // remove from cache
                self.base
                    .cachedb
                    .remove(format!("{}:{}", self.options.table_pastes.prefix, url))
                    .await;

                // return
                return Ok(());
            }
            Err(_) => return Err(PasteError::Other),
        };
    }

    /// Edit an existing paste's metadata by `url`
    ///
    /// # Arguments
    /// * `url` - the paste to edit
    /// * `password` - the paste's edit password
    /// * `metadata` - the new metadata of the paste
    /// * `editing_as` - the userstate of the user we're editing the paste as
    pub async fn edit_paste_metadata_by_url(
        &self,
        mut url: String,
        password: String,
        metadata: PasteMetadata,
        editing_as: Option<FullUser<UserMetadata>>,
    ) -> Result<()> {
        url = idna::punycode::encode_str(&url).unwrap().to_lowercase();

        if url.ends_with("-") {
            url.pop();
        }

        // get paste
        let existing = match self.get_paste_by_url(url.clone()).await {
            Ok(p) => p,
            Err(err) => return Err(err),
        };

        // check password
        let mut skip_password_check: bool = false;

        if let Some(ua) = editing_as {
            // check if we're the paste owner
            if ua.user.username == existing.metadata.owner {
                skip_password_check = true;
            }
            // check if we have the "ManagePastes" permission
            else if ua.level.permissions.contains(&"ManagePastes".to_string()) {
                skip_password_check = true;
            }
        }

        if skip_password_check == false {
            if utility::hash(password) != existing.password {
                return Err(PasteError::PasswordIncorrect);
            }
        }

        // edit paste
        let query: String = if (self.base.db._type == "sqlite") | (self.base.db._type == "mysql") {
            "UPDATE \":t\" SET \":metadata\" = ? WHERE \":url\" = ?"
        } else {
            "UPDATE \":t\" SET (\":metadata\" = $1) WHERE \":url\" = $2"
        }
        .to_string()
        .replace(":t", &self.options.table_pastes.table_name)
        .replace(":url", &self.options.table_pastes.url)
        .replace(":metadata", &self.options.table_pastes.metadata);

        let c = &self.base.db.client;
        match sqlquery(&query)
            .bind::<&String>(match serde_json::to_string(&metadata) {
                Ok(ref m) => m,
                Err(_) => return Err(PasteError::ValueError),
            })
            .bind::<&String>(&url)
            .execute(c)
            .await
        {
            Ok(_) => {
                // remove from cache
                self.base
                    .cachedb
                    .remove(format!("{}:{}", self.options.table_pastes.prefix, url))
                    .await;

                // return
                return Ok(());
            }
            Err(_) => return Err(PasteError::Other),
        };
    }

    // views

    /// Get an existing url's view count
    ///
    /// # Arguments
    /// * `url` - the paste to count the view for
    pub async fn get_views_by_url(&self, mut url: String) -> i32 {
        url = idna::punycode::encode_str(&url).unwrap().to_lowercase();

        if url.ends_with("-") {
            url.pop();
        }

        // get views
        match self
            .base
            .cachedb
            .get(format!("{}:{}", self.options.table_views.prefix, url))
            .await
        {
            Some(c) => c.parse::<i32>().unwrap(),
            None => {
                // try to count from "views"
                if self.options.view_mode == ViewMode::AuthenticatedOnce {
                    let query: String =
                        if (self.base.db._type == "sqlite") | (self.base.db._type == "mysql") {
                            "SELECT * FROM \":t\" WHERE \"url\" = ?"
                        } else {
                            "SELECT * FROM \":t\" WHERE \"url\" = $1"
                        }
                        .to_string()
                        .replace(":t", &self.options.table_views.table_name);

                    let c = &self.base.db.client;
                    match sqlquery(&query).bind::<&String>(&url).fetch_all(c).await {
                        Ok(views) => {
                            let views = views.len();

                            // store in cache
                            self.base
                                .cachedb
                                .set(
                                    format!("{}:{}", self.options.table_views.prefix, url),
                                    views.to_string(),
                                )
                                .await;

                            // return
                            return views as i32;
                        }
                        Err(_) => return 0,
                    };
                }

                // return 0 by default
                0
            }
        }
    }

    /// Update an existing url's view count
    ///
    /// # Arguments
    /// * `url` - the paste to count the view for
    /// * `as_user` - the userstate of the user viewing this (for [`ViewMode::AuthenticatedOnce`])
    pub async fn incr_views_by_url(
        &self,
        mut url: String,
        as_user: Option<FullUser<UserMetadata>>,
    ) -> Result<()> {
        url = idna::punycode::encode_str(&url).unwrap().to_lowercase();

        if url.ends_with("-") {
            url.pop();
        }

        // handle AuthenticatedOnce
        if self.options.view_mode == ViewMode::AuthenticatedOnce {
            match as_user {
                Some(ua) => {
                    // check for view
                    if self
                        .user_has_viewed_paste(url.clone(), ua.user.username.clone())
                        .await
                    {
                        // can only view once in this mode
                        return Ok(());
                    }

                    // create view
                    let query: String =
                        if (self.base.db._type == "sqlite") | (self.base.db._type == "mysql") {
                            "INSERT INTO \":t\" VALUES (?, ?)"
                        } else {
                            "INSERT INTO \":t\" VALEUS ($1, $2)"
                        }
                        .to_string()
                        .replace(":t", &self.options.table_views.table_name);

                    let c = &self.base.db.client;
                    match sqlquery(&query)
                        .bind::<&String>(&url)
                        .bind::<&String>(&ua.user.username)
                        .execute(c)
                        .await
                    {
                        Ok(_) => (), // do nothing so cache is incremented
                        Err(_) => return Err(PasteError::Other),
                    };
                }
                None => return Ok(()), // not technically an error, just not allowed
            }
        }

        // add view
        // views never reach the database, they're only stored in memory
        match self
            .base
            .cachedb
            .incr(format!("{}:{}", self.options.table_views.prefix, url))
            .await
        {
            // swapped for some reason??
            false => Ok(()),
            true => Err(PasteError::Other),
        }
    }

    /// Check if a user has views a paste given the `url` and their `username`
    ///
    /// # Arguments
    /// * `url` - the paste url
    /// * `username` - the username of the user
    pub async fn user_has_viewed_paste(&self, url: String, username: String) -> bool {
        if self.options.view_mode == ViewMode::AuthenticatedOnce {
            let query: String =
                if (self.base.db._type == "sqlite") | (self.base.db._type == "mysql") {
                    "SELECT * FROM \":t\" WHERE \"url\" = ? AND \"username\" = ?"
                } else {
                    "SELECT * FROM \":t\" WHERE \"url\" = $1 AND \"username\" = ?"
                }
                .to_string()
                .replace(":t", &self.options.table_views.table_name);

            let c = &self.base.db.client;
            match sqlquery(&query)
                .bind::<&String>(&url)
                .bind::<&String>(&username)
                .fetch_one(c)
                .await
            {
                Ok(_) => return true,
                Err(_) => return false,
            };
        }

        false
    }
}
