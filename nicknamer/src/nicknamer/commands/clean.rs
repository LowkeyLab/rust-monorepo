use linkify::{LinkFinder, LinkKind};
use log::error;

pub trait UrlCleaner {
    fn clean_url(&mut self, msg: &str) -> String;
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

impl UrlCleaner for UrlCleanerImpl {
    fn clean_url(&mut self, msg: &str) -> String {
        let finder = &mut self.link_finder;
        for link in finder.kinds(&[LinkKind::Url]).links(msg) {
            let Ok(cleaned) = self.cleaner.clear_single_url_str(link.as_str()) else {
                error!("Failed to clean url: {}", link.as_str());
            };
        }
        todo!();
    }
}
