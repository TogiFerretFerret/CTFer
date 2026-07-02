use serde::{Deserialize, Serialize};

/// A string holding rendered/trusted HTML (e.g. a challenge description body).
/// The newtype stops raw user input from being mistaken for sanitized HTML at
/// the type level.
///
/// ```
/// use cctf_rs::libs::types::htmlstring::HtmlString;
///
/// let body = HtmlString("<p>welcome to cctf.rs</p>".to_string());
/// assert_eq!(body.0, "<p>welcome to cctf.rs</p>");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HtmlString(pub String);
