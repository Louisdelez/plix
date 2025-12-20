# Maintainers

This document lists the maintainers of the Plix project and their responsibilities.

## Lead Maintainer

| Name | GitHub | Role | Focus Areas |
|------|--------|------|-------------|
| Louis de Lez | [@Louisdelez](https://github.com/Louisdelez) | Lead | Architecture, Core Systems |

## Responsibilities

### Lead Maintainer

- Overall project direction and roadmap
- Final approval on significant changes
- Release management and version tagging
- Security response coordination
- Community management

### Maintainers

- Review and merge pull requests
- Triage issues
- Maintain documentation
- Support community contributors
- Enforce code quality standards

## Release Signing

### GPG Key for Releases

Releases are signed with GPG for verification. The signing key will be published here once v1.0.0 is released.

**Key Fingerprint**: (To be added before v1.0.0 release)

To verify a release:

```bash
# Import the maintainer's public key
gpg --keyserver keys.openpgp.org --recv-keys <KEY_FINGERPRINT>

# Verify a signed tag
git verify-tag v1.0.0

# Verify a signed commit
git verify-commit <COMMIT_SHA>
```

### Key Rotation Policy

- Release signing keys are rotated annually
- New keys are announced via GitHub Security Advisory
- Old keys are revoked and removed from keyservers
- A 30-day overlap period allows verification of recent releases

## Becoming a Maintainer

Maintainer status is granted to contributors who have:

1. Made significant, high-quality contributions over time
2. Demonstrated deep understanding of the codebase
3. Shown commitment to the project's values and direction
4. Actively participated in code review and issue triage
5. Exhibited good judgment and collaboration skills

If you're interested in becoming a maintainer, start by:
- Contributing code and documentation
- Reviewing pull requests
- Helping with issue triage
- Participating in discussions

## Emeritus Maintainers

Maintainers who have stepped back from active involvement but made significant contributions.

(None yet)

## Contact

For maintainer-related inquiries:
- Open a GitHub Discussion with the "Maintainers" label
- For sensitive matters, email: maintainers@plix.dev

For security issues, see [SECURITY.md](SECURITY.md).
