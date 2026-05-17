pub mod environment_resolver;
pub mod error;
pub mod keychain_resolver;
pub mod memory_resolver;
pub mod redaction;
pub mod resolver;
pub mod secrets_store;

pub use environment_resolver::EnvironmentResolver;
pub use error::SecretError;
pub use keychain_resolver::KeychainResolver;
pub use memory_resolver::MemoryResolver;
pub use redaction::{Redaction, redact};
pub use resolver::SecretResolver;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secret_resolver_trait_is_object_safe() {
        let _: Box<dyn SecretResolver> = Box::new(MemoryResolver::new());
    }

    #[test]
    fn memory_resolver_resolves_inserted_values() {
        let resolver = MemoryResolver::new();
        resolver.insert("api_key", "sk-live-12345");
        resolver.insert("db_password", "hunter2");

        assert_eq!(resolver.resolve("api_key").unwrap(), "sk-live-12345");
        assert_eq!(resolver.resolve("db_password").unwrap(), "hunter2");
    }

    #[test]
    fn memory_resolver_returns_not_found_for_missing_key() {
        let resolver = MemoryResolver::new();
        let err = resolver.resolve("missing_key").unwrap_err();
        assert_eq!(err, SecretError::NotFound("missing_key".to_string()));
    }

    #[test]
    fn memory_resolver_remove_clears_entry() {
        let resolver = MemoryResolver::new();
        resolver.insert("tmp", "value");
        assert_eq!(resolver.resolve("tmp").unwrap(), "value");

        resolver.remove("tmp");
        assert!(resolver.resolve("tmp").is_err());
    }

    #[test]
    fn memory_resolver_clear_removes_all() {
        let resolver = MemoryResolver::new();
        resolver.insert("a", "1");
        resolver.insert("b", "2");
        resolver.clear();

        assert!(resolver.resolve("a").is_err());
        assert!(resolver.resolve("b").is_err());
    }

    #[test]
    fn environment_resolver_reads_env_var() {
        let resolver = EnvironmentResolver::new();
        // This variable is set by cargo test infrastructure
        assert!(resolver.resolve("CARGO_MANIFEST_DIR").is_ok());
    }

    #[test]
    fn environment_resolver_returns_error_for_missing_var() {
        let resolver = EnvironmentResolver::new();
        let err = resolver
            .resolve("OPENSPACE_NONEXISTENT_VAR_12345")
            .unwrap_err();
        assert_eq!(
            err,
            SecretError::EnvVarNotSet("OPENSPACE_NONEXISTENT_VAR_12345".to_string())
        );
    }

    #[test]
    fn keychain_resolver_compiles_and_is_no_op_on_non_macos() {
        let resolver = KeychainResolver::default();
        let result = resolver.resolve("any_key");

        #[cfg(not(target_os = "macos"))]
        {
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(
                err.to_string().contains("no-op"),
                "Expected no-op error on non-macOS: {err}"
            );
        }

        // On macOS we just verify it compiles and returns a result;
        // the actual keychain availability is not required by the acceptance criteria.
        #[cfg(target_os = "macos")]
        {
            // Result may be Ok or Err depending on keychain availability,
            // but it must not panic.
            let _ = result;
        }
    }

    #[test]
    fn redaction_strips_expected_patterns() {
        let raw = "Authorization: Bearer supersecrettoken";
        let cleaned = redact(raw);
        assert_eq!(cleaned, "Authorization: Bearer [REDACTED]");
    }

    #[test]
    fn secret_error_uses_thiserror() {
        let err = SecretError::NotFound("my_key".to_string());
        let display = err.to_string();
        assert!(display.contains("my_key"));
    }
}
