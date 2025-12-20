# Security Policy

## Supported Versions

We release patches for security vulnerabilities in the following versions:

| Version | Supported          |
| ------- | ------------------ |
| 1.0.x   | :white_check_mark: |
| < 1.0   | :x:                |

## Reporting a Vulnerability

The Plix team takes security vulnerabilities seriously. We appreciate your efforts to responsibly disclose your findings.

### How to Report

**Please do NOT report security vulnerabilities through public GitHub issues.**

Instead, please report them via email to: **security@plix.dev**

Include the following information in your report:

1. **Description**: A clear description of the vulnerability
2. **Impact**: What could an attacker achieve?
3. **Reproduction Steps**: Detailed steps to reproduce the issue
4. **Affected Versions**: Which versions are affected?
5. **Suggested Fix**: (Optional) If you have a fix in mind

### What to Expect

- **Acknowledgment**: We will acknowledge receipt of your report within 48 hours
- **Initial Assessment**: Within 1 week, we will provide an initial assessment
- **Updates**: We will keep you informed of our progress
- **Resolution**: We aim to resolve critical vulnerabilities within 30 days
- **Credit**: We will credit you in the security advisory (unless you prefer to remain anonymous)

### Disclosure Policy

- We follow a coordinated disclosure process
- We will work with you to understand and resolve the issue
- Once a fix is available, we will publish a security advisory
- We request that you do not disclose the vulnerability publicly until we have released a fix

## Security Scope

### In Scope

- **Server Binary** (`plix-server`, `plix-server-headless`)
  - Network protocol vulnerabilities
  - Authentication/authorization bypasses
  - Denial of service vulnerabilities
  - Memory safety issues
  - Privilege escalation

- **Client Binary** (`plix-client`)
  - Malicious server attacks on client
  - Memory safety issues in rendering/input handling
  - Local file access vulnerabilities

- **Mod Runtime** (`plix-mod-runtime-wasm`)
  - Sandbox escapes
  - Resource exhaustion attacks
  - Host function vulnerabilities

### Out of Scope

- Social engineering attacks
- Physical access attacks
- Vulnerabilities in third-party dependencies (report these upstream)
- Issues in pre-release or development builds
- Self-DoS (crashing your own client/server)
- Cheat development or game balance issues

## Security Best Practices

### For Server Operators

1. **Keep Updated**: Always run the latest stable version
2. **Firewall Rules**: Only expose the game port (default: 7777/UDP)
3. **Rate Limiting**: Use the built-in rate limiting features
4. **Mod Vetting**: Only install mods from trusted sources
5. **Backups**: Maintain regular backups of world data

### For Players

1. **Download from Official Sources**: Only download Plix from GitHub Releases
2. **Verify Checksums**: Check SHA-256 checksums match
3. **Be Cautious with Mods**: Only install mods from trusted sources

### For Mod Developers

1. **Minimize Permissions**: Only request capabilities your mod needs
2. **Validate Input**: Never trust data from players
3. **Resource Limits**: Respect CPU and memory budgets
4. **No Sensitive Data**: Never store credentials or secrets in mods

## Security Features

Plix includes several security features:

- **Server-Authoritative Architecture**: Clients cannot directly modify game state
- **Rate Limiting**: Protection against packet flooding
- **Input Validation**: All client inputs are validated server-side
- **WASM Sandbox**: Mods run in an isolated WebAssembly sandbox
- **Capability System**: Mods must declare and request specific permissions

## Past Security Advisories

None yet. This section will be updated when security advisories are published.

## Contact

- **Security Reports**: security@plix.dev
- **General Questions**: See [SUPPORT.md](SUPPORT.md)
