# Plix Roadmap

This document outlines the planned development direction for Plix.

## Versioning Policy

Plix follows [Semantic Versioning 2.0.0](https://semver.org/):

- **MAJOR** (1.x.x → 2.x.x): Breaking changes to protocol, mod API, or save formats
- **MINOR** (1.0.x → 1.1.x): New features, backward-compatible
- **PATCH** (1.0.0 → 1.0.1): Bug fixes, no new features

## Current Release: v1.0.0

The first stable release of Plix, providing:

- Complete multiplayer voxel game platform
- Server-authoritative architecture with anti-cheat
- Full modding support (data mods, WASM script mods)
- Cross-platform support (Windows, Linux, macOS)
- Content system (quests, NPCs, dungeons)
- Production-ready performance and security

## v1.0.x Maintenance

**Focus**: Stability, security, and polish

### Planned

- Security patch releases as needed
- Critical bug fixes
- Performance optimizations
- Documentation improvements

### Not Included

- New features
- Breaking changes
- Major UI overhauls

## v1.1.x Feature Release

**Focus**: Quality of life and expanded content

### Planned Features

- **Enhanced UI/UX**
  - Improved settings menus
  - Better keybind editor
  - Accessibility improvements

- **Content Expansion**
  - Additional quest types
  - More dungeon templates
  - Expanded crafting recipes

- **Modding Enhancements**
  - Additional mod API hooks
  - Improved mod debugging tools
  - Better mod compatibility reporting

- **Server Administration**
  - Enhanced admin commands
  - Improved logging and monitoring
  - Better player management tools

### Timeline

- Development: Q2-Q3 2025
- Release: Q3 2025

## v2.0.0 Major Release

**Focus**: Next-generation features requiring breaking changes

### Under Consideration

- **Protocol V2**
  - Improved compression
  - Enhanced security features
  - Better mobile support

- **Mod API V2**
  - Async event system
  - Enhanced capabilities
  - Improved sandboxing

- **Content Schema V2**
  - Richer entity relationships
  - Dynamic content generation
  - Procedural quest system

- **Architecture Improvements**
  - Multi-threaded server
  - Improved chunk streaming
  - Better memory efficiency

### Migration

- Automatic migration tools for v1.x → v2.0
- Extended support for v1.x during transition
- Clear deprecation warnings in v1.x

### Timeline

- Design: Q4 2025
- Development: 2026
- Release: 2026

## Long-Term Vision

### 2025-2026

- Establish Plix as a stable, modding-friendly platform
- Build active modding community
- Expand official content
- Improve performance and scalability

### Beyond 2026

- Mobile platform support (tentative)
- VR/XR exploration (tentative)
- Advanced procedural generation
- Community-driven development

## Contributing to the Roadmap

### Feature Requests

Open a [GitHub Discussion](https://github.com/Louisdelez/plix/discussions) with:

1. Clear description of the feature
2. Use cases and benefits
3. Potential implementation considerations

### Community Input

The roadmap is influenced by:

- GitHub Issues and Discussions
- Community feedback
- Technical feasibility
- Maintainer capacity

### No Promises

This roadmap represents current intentions, not commitments. Priorities may shift based on community needs, technical discoveries, or resource availability.

## Stability Guarantees

### v1.x Series

- **Protocol**: v1.x clients work with v1.x servers (same major version)
- **Mod API**: v1.x mods work with v1.x engine (same major version)
- **Save Data**: v1.x saves work with v1.x versions (migration may be needed)
- **Configuration**: v1.x configs work with v1.x versions (migration may be needed)

### Deprecation Policy

1. Features deprecated in v1.x will be removed in v2.0
2. Deprecation warnings appear at least one minor version before removal
3. Migration guides provided for all breaking changes

## Version History

| Version | Date | Highlights |
|---------|------|------------|
| 1.0.0 | 2025-12-XX | First stable release |

See [CHANGELOG.md](../CHANGELOG.md) for detailed release notes.
