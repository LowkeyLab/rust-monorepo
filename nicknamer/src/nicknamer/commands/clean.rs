use linkify::LinkFinder;

pub trait UrlCleaner {
    /// Cleans a message by removing tracking parameters from URLs within it.
    /// # Returns
    /// The cleaned message with original and cleaned URLs.
    fn clean_message(&mut self, msg: &str) -> CleanedMessage;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CleanedMessage {
    message: String,
    cleaned_urls: Vec<(String, String)>,
}

impl CleanedMessage {
    /// Creates a new CleanedMessage with the given message and no cleaned URLs.
    pub fn new(message: String) -> Self {
        Self {
            message,
            cleaned_urls: Vec::new(),
        }
    }

    /// Adds a cleaned URL to the list of cleaned URLs.
    pub fn add_cleaned_url(&mut self, original: String, cleaned: String) {
        self.cleaned_urls.push((original, cleaned));
    }

    /// Returns the cleaned message.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns the list of cleaned URLs.
    pub fn cleaned_urls(&self) -> &[(String, String)] {
        &self.cleaned_urls
    }
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
    fn clean_message(&mut self, msg: &str) -> CleanedMessage {
        let mut result = msg.to_string();
        let mut cleaned_message = CleanedMessage::new(msg.to_string());

        // Find all links in the message
        let links: Vec<_> = self.link_finder.links(msg).collect();

        // If no links found, return the original message
        if links.is_empty() {
            return cleaned_message;
        }

        // Process each link
        for link in links {
            let original_url = link.as_str();

            // Clean the URL using the clearurls crate's functionality
            if let Ok(cleaned_url) = self.cleaner.clear_single_url_str(original_url) {
                // Convert Cow<str> to String for comparison
                let cleaned_url_str = cleaned_url.into_owned();

                // Replace the original URL with the cleaned one if they differ
                if original_url != cleaned_url_str {
                    result = result.replace(original_url, &cleaned_url_str);
                    cleaned_message.add_cleaned_url(original_url.to_string(), cleaned_url_str);
                }
            }
        }

        // Create a new CleanedMessage with the updated message and the tracked URLs
        let mut final_message = CleanedMessage::new(result);
        for (original, cleaned) in cleaned_message.cleaned_urls() {
            final_message.add_cleaned_url(original.clone(), cleaned.clone());
        }
        final_message
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_message_with_no_urls() {
        // Arrange
        let mut cleaner = UrlCleanerImpl::new();
        let message = "This is a message with no URLs";

        // Act
        let result = cleaner.clean_message(message);

        // Assert
        assert_eq!(result.message(), message);
        assert!(result.cleaned_urls().is_empty());
    }

    #[test]
    fn test_clean_message_with_empty_message() {
        // Arrange
        let mut cleaner = UrlCleanerImpl::new();
        let message = "";

        // Act
        let result = cleaner.clean_message(message);

        // Assert
        assert_eq!(result.message(), message);
        assert!(result.cleaned_urls().is_empty());
    }

    #[test]
    fn test_clean_message_with_url_that_needs_cleaning() {
        // Arrange
        let mut cleaner = UrlCleanerImpl::new();
        let original_url = "https://example.com/page?utm_source=test&utm_medium=email";
        let message = format!("Check out this link: {}", original_url);

        // Act
        let result = cleaner.clean_message(&message);

        // Assert
        assert_ne!(result.message(), message);
        assert!(result.message().contains("https://example.com/page"));
        assert!(!result.message().contains("utm_source=test"));

        // Check that cleaned_urls contains the original and cleaned URLs
        assert!(!result.cleaned_urls().is_empty());
        let has_cleaned_url = result.cleaned_urls().iter().any(|(orig, cleaned)| {
            orig.contains("utm_source=test") && !cleaned.contains("utm_source=test")
        });
        assert!(
            has_cleaned_url,
            "Cleaned URLs should contain the original and cleaned URL"
        );
    }

    #[test]
    fn test_clean_message_with_url_that_doesnt_need_cleaning() {
        // Arrange
        let mut cleaner = UrlCleanerImpl::new();
        let message = "Check out this link: https://example.com/page";

        // Act
        let result = cleaner.clean_message(message);

        // Assert
        assert_eq!(result.message(), message);
        assert!(result.cleaned_urls().is_empty());
    }

    #[test]
    fn test_clean_message_with_multiple_urls() {
        // Arrange
        let mut cleaner = UrlCleanerImpl::new();
        let message = "Check out these links: https://example.com/page?utm_source=test and https://another-example.com/page";

        // Act
        let result = cleaner.clean_message(message);

        // Assert
        assert_ne!(result.message(), message);
        assert!(result.message().contains("https://example.com/page"));
        assert!(!result.message().contains("utm_source=test"));
        assert!(
            result
                .message()
                .contains("https://another-example.com/page")
        );

        // Check that cleaned_urls contains the original and cleaned URLs
        assert!(!result.cleaned_urls().is_empty());
        let has_cleaned_url = result.cleaned_urls().iter().any(|(orig, cleaned)| {
            orig.contains("utm_source=test") && !cleaned.contains("utm_source=test")
        });
        assert!(
            has_cleaned_url,
            "Cleaned URLs should contain the original and cleaned URL"
        );
    }
}
