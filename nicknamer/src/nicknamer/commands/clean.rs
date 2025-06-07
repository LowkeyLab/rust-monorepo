use linkify::LinkFinder;

pub trait UrlCleaner {
    /// Cleans a message by removing tracking parameters from URLs within it.
    /// # Returns
    /// The cleaned message.
    fn clean_message(&mut self, msg: &str) -> String;
}

pub struct UrlCleanerImpl {
    link_finder: LinkFinder,
    cleaner: clearurls::UrlCleaner,
}

impl UrlCleanerImpl {
    pub fn new() -> Self {
        Self {
            link_finder: LinkFinder::new(),
            cleaner: clearurls::UrlCleaner::from_embedded_rules().unwrap(),
        }
    }
}
