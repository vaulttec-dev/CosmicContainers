//! Core logic for linux-pop-extension.

/// Returns the default greeting message.
pub fn greeting() -> &'static str {
    "Hello, world!"
}

#[cfg(test)]
mod tests {
    use super::greeting;

    #[test]
    fn greeting_is_non_empty() {
        assert!(!greeting().is_empty());
    }
}
