#[derive(Default, PartialEq)]
pub struct Paste {
    // selectors
    pub custom_url: String,
    pub id: String,
    // passwords
    pub edit_password: String,
    // dates
    pub pub_date: i64,
    pub edit_date: i64,
    // ...
    pub content: String,
    pub metadata: PasteMetadata,
}

#[derive(Default, PartialEq)]
pub struct PasteMetadata {
    pub owner: String,
}

// ...
pub fn create_database() {
    return;
}

pub fn create_dummy(mut custom_url: Option<&str>) -> Paste {
    if custom_url.is_none() {
        custom_url = Option::Some("dummy_paste");
    }

    return Paste {
        custom_url: custom_url.unwrap().to_string(),
        id: "".to_string(),
        // passwords
        edit_password: "".to_string(),
        // dates
        pub_date: 0,
        edit_date: 0,
        // ...
        content: "".to_string(),
        metadata: PasteMetadata {
            owner: "".to_string()
        },
    }
}