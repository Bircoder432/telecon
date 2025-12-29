#[derive(Debug, Clone)]
pub struct Notification {
    pub text: Option<String>,
    pub files: Vec<String>,
    pub media: Vec<String>,
    pub buttons: Vec<(String, String)>,
}
