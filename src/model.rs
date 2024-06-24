use dorsal::DefaultReturn;

/// General API errors
pub enum DatabaseError {
    MustBeUnique,
    NotAllowed,
    ValueError,
    NotFound,
    Other,
}

impl DatabaseError {
    pub fn to_string(&self) -> String {
        use DatabaseError::*;
        match self {
            MustBeUnique => {
                String::from("One or more of the given values must be unique but is not.")
            }
            NotAllowed => String::from("You are not allowed to access this resource."),
            ValueError => String::from("One of the field values given is invalid."),
            NotFound => String::from("No asset with this selector could be found."),
            _ => String::from("An unspecified error has occured"),
        }
    }
}

impl<T: Default> Into<DefaultReturn<T>> for DatabaseError {
    fn into(self) -> DefaultReturn<T> {
        DefaultReturn {
            success: false,
            message: self.to_string(),
            payload: T::default(),
        }
    }
}
