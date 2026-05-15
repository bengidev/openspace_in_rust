# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| main    | :white_check_mark: |
| < 0.1.0 | :x:                |

## Reporting a Vulnerability

If you discover a security vulnerability in OpenSpace, please report it responsibly.

**Do not open a public issue.**

Instead, please email the maintainer directly or open a private security advisory through GitHub.

We will:
- Acknowledge receipt within 48 hours
- Investigate and validate the vulnerability
- Provide a timeline for a fix
- Coordinate disclosure once the issue is resolved

## Security Considerations

OpenSpace integrates with AI models and desktop environments. Key security areas:

- **Local Model Execution**: Local AI models run on your machine. Ensure you trust the models you download.
- **API Keys**: Never commit API keys or credentials to the repository. Use environment variables or secure key storage.
- **Desktop Permissions**: OpenSpace may require elevated permissions for desktop integration. Review requested permissions carefully.
- **Subagent Isolation**: Subagents execute tasks on behalf of the user. The core engine isolates subagent operations to prevent unauthorized system access.

## Best Practices

- Keep your Rust toolchain and dependencies up to date
- Run OpenSpace with the least privileges necessary
- Audit third-party AI model integrations before use
- Report suspicious behavior immediately
