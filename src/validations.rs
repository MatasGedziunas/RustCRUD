use regex::Regex;

pub struct Validations {}

impl Validations {
    pub fn validate(field: &str) -> Result<(), warp::Rejection> {
        let max_count = 15;
        let min_count = 4;
        if (Self::bigger_than_max_charachters(field, max_count)) {
            return Err(warp::reject::custom(TooLongError));
        } else if (Self::smaller_than_min_charachters(field, min_count)) {
            return Err(warp::reject::custom(TooShortError));
        } else {
            return Ok(());
        }
    }

    pub fn bigger_than_max_charachters(field: &str, count: usize) -> bool {
        if (field.len() > count) {
            return true;
        }
        return false;
    }
    pub fn smaller_than_min_charachters(field: &str, count: usize) -> bool {
        if (field.len() < count) {
            return true;
        }
        return false;
    }
    pub fn no_special_charachters(field: &str) -> bool {
        // Define a regex pattern that matches non-alphanumeric characters
        let pattern = Regex::new(r"[^a-zA-Z0-9]").unwrap();

        // Use the regex pattern to find matches in the input string
        pattern.is_match(field)
    }
}

#[derive(Debug)]
pub struct TooShortError;
impl warp::reject::Reject for TooShortError {}

#[derive(Debug)]
pub struct TooLongError;
impl warp::reject::Reject for TooLongError {}

#[derive(Debug)]
pub struct SpecialCharachtersNotAllowed;
impl warp::reject::Reject for SpecialCharachtersNotAllowed {}

#[derive(Debug)]
pub struct DatabaseError;
impl warp::reject::Reject for DatabaseError {}

#[derive(Debug)]
pub struct MissingParameter;
impl warp::reject::Reject for MissingParameter {}

#[derive(Debug)]
pub struct NoMatchingUser;
impl warp::reject::Reject for NoMatchingUser {}
