/// State for each open document
#[derive(Debug)]
pub struct DocumentState {
    pub content: String,
    #[allow(dead_code)]
    pub flavor_name: Option<String>, // Detected from modeline or default - will be used for per-document flavor selection
}
