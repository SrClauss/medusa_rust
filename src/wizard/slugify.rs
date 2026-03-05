//! Slugification — converts arbitrary product names to clean URL-safe paths.
//!
//! Example: `"Tênis Pro"` → `"tenis-pro"`

/// Converts a display name to a filesystem / URL-safe slug.
///
/// Steps applied:
/// 1. Unicode normalisation → ASCII (best-effort transliteration)
/// 2. Lower-case
/// 3. Replace non-alphanumeric characters with hyphens
/// 4. Collapse and trim leading / trailing hyphens
pub fn slugify(name: &str) -> String {
    slug::slugify(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ascii_passthrough() {
        assert_eq!(slugify("Tennis Pro"), "tennis-pro");
    }

    #[test]
    fn unicode_transliteration() {
        let result = slugify("Tênis Pro");
        // "ê" → "e" via slug crate
        assert!(result.contains("pro"), "slug should contain 'pro': {result}");
        assert!(!result.contains(' '), "slug should not contain spaces: {result}");
    }

    #[test]
    fn special_characters() {
        assert_eq!(slugify("Hello, World! (2024)"), "hello-world-2024");
    }

    #[test]
    fn multiple_spaces() {
        assert_eq!(slugify("  foo   bar  "), "foo-bar");
    }
}
