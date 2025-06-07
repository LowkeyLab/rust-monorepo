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

impl UrlCleaner for UrlCleanerImpl {
    fn clean_message(&mut self, msg: &str) -> String {
        let mut result = msg.to_string();

        // Find all links in the message
        let links: Vec<_> = self.link_finder.links(msg).collect();

        // If no links found, return the original message
        if links.is_empty() {
            return result;
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
                }
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    use mockall::*;

    // Create a mock for the UrlCleaner trait
    mock! {
        UrlCleaner {}
        impl UrlCleaner for UrlCleaner {
            fn clean_message(&mut self, msg: &str) -> String;
        }
    }

    #[test]
    fn test_clean_message_with_no_urls() {
        // Arrange
        let mut cleaner = UrlCleanerImpl::new();
        let message = "This is a message with no URLs";

        // Act
        let result = cleaner.clean_message(message);

        // Assert
        assert_eq!(result, message);
    }

    #[test]
    fn test_clean_message_with_empty_message() {
        // Arrange
        let mut cleaner = UrlCleanerImpl::new();
        let message = "";

        // Act
        let result = cleaner.clean_message(message);

        // Assert
        assert_eq!(result, message);
    }

    #[test]
    fn test_clean_message_with_url_that_needs_cleaning() {
        // Arrange
        let mut cleaner = UrlCleanerImpl::new();
        let message =
            "Check out this link: https://example.com/page?utm_source=test&utm_medium=email";

        // We'll mock the behavior of clear_single_url_str
        // Since we can't easily mock the clearurls crate, we'll test the integration
        // by verifying that the URL is different after cleaning

        // Act
        let result = cleaner.clean_message(message);

        // Assert
        assert_ne!(result, message);
        assert!(result.contains("https://example.com/page"));
        assert!(!result.contains("utm_source=test"));
    }

    #[test]
    fn test_clean_message_with_url_that_doesnt_need_cleaning() {
        // Arrange
        let mut cleaner = UrlCleanerImpl::new();
        let message = "Check out this link: https://example.com/page";

        // Act
        let result = cleaner.clean_message(message);

        // Assert
        assert_eq!(result, message);
    }

    #[test]
    fn test_clean_message_with_multiple_urls() {
        // Arrange
        let mut cleaner = UrlCleanerImpl::new();
        let message = "Check out these links: https://example.com/page?utm_source=test and https://another-example.com/page";

        // Act
        let result = cleaner.clean_message(message);

        // Assert
        assert_ne!(result, message);
        assert!(result.contains("https://example.com/page"));
        assert!(!result.contains("utm_source=test"));
        assert!(result.contains("https://another-example.com/page"));
    }
}
