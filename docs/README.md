# Documentation

Welcome to the **prompt-compress** documentation! This directory contains comprehensive guides for users, developers, and integrators.

## Getting Started

- [QUICKSTART.md](./QUICKSTART.md) - Quick start guide to get up and running in minutes
- [README.md](../README.md) - Main project README with overview and installation

## Architecture & Specification

- [CLAUDE.md](./CLAUDE.md) - Complete project specification and design document
- [CONSOLIDATED-ARCHITECTURE.md](./CONSOLIDATED-ARCHITECTURE.md) - Detailed architecture overview with database-backed pattern system
- [CONSOLIDATION-SUMMARY.md](./CONSOLIDATION-SUMMARY.md) - Summary of architecture consolidation (v0.4)

## Developer Guides

- [CONTRIBUTORS.md](./CONTRIBUTORS.md) - **Comprehensive developer guide** covering:
  - Project architecture and codebase structure
  - Database schema and Wikidata integration
  - Adding new language pairs (Q16917)
  - CLI tools and REST API
  - Pattern system and HITL feedback
  - Testing and development workflow

## Integration Guides

- [MCP-SERVER-GUIDE.md](./MCP-SERVER-GUIDE.md) - **Build MCP servers** with prompt-compress:
  - What is Model Context Protocol (MCP)
  - Creating MCP servers for Claude Desktop
  - Exposing tools, resources, and prompts
  - Advanced features and deployment
  - Complete working examples

- [CLAUDE-SKILLS-GUIDE.md](./CLAUDE-SKILLS-GUIDE.md) - **Create Claude skills** powered by prompt-compress:
  - What are Claude skills
  - Building reusable optimization skills
  - Context-aware optimization
  - Skill composition and testing
  - Real-world examples

## Testing & Validation

- [TESTING-STRATEGY.md](./TESTING-STRATEGY.md) - **Comprehensive testing strategy** with Hugging Face datasets:
  - Two-tier testing approach (Q1678 + HF datasets)
  - Real-world prompt datasets (WildChat, UltraChat, Alpaca)
  - Benchmarking and regression detection
  - CI/CD integration
  - Atlas training from HF data

## Test Results & Verification

- [PHASE3-COMPLETE.md](./PHASE3-COMPLETE.md) - Phase 3 implementation and test results
- [TEST-RESULTS.md](./TEST-RESULTS.md) - Comprehensive test results and verification
- [VERIFICATION-REPORT.md](./VERIFICATION-REPORT.md) - Detailed verification report
- [FINAL-SUMMARY.md](./FINAL-SUMMARY.md) - Complete project summary with metrics

## Feature Documentation

- [AGGRESSIVE-MODE-SUMMARY.md](./AGGRESSIVE-MODE-SUMMARY.md) - Aggressive mode documentation (70-85% savings)
- [AGENTS.md](./AGENTS.md) - Agent-related documentation

## Quick Links by Task

### I want to...

#### Use prompt-compress
→ Start with [QUICKSTART.md](./QUICKSTART.md)

#### Understand the architecture
→ Read [CONSOLIDATED-ARCHITECTURE.md](./CONSOLIDATED-ARCHITECTURE.md)

#### Contribute code
→ Follow [CONTRIBUTORS.md](./CONTRIBUTORS.md)

#### Add new language pairs (Wikidata concepts)
→ See [CONTRIBUTORS.md - Adding New Language Pairs](./CONTRIBUTORS.md#adding-new-language-pairs-q16917)

#### Build an MCP server
→ Follow [MCP-SERVER-GUIDE.md](./MCP-SERVER-GUIDE.md)

#### Create Claude skills
→ Follow [CLAUDE-SKILLS-GUIDE.md](./CLAUDE-SKILLS-GUIDE.md)

#### Add new optimization patterns
→ See [CONTRIBUTORS.md - Pattern System](./CONTRIBUTORS.md#pattern-system)

#### Deploy the API server
→ See [CONTRIBUTORS.md - REST API](./CONTRIBUTORS.md#rest-api)

#### Understand the database schema
→ See [CONTRIBUTORS.md - Database Schema](./CONTRIBUTORS.md#database-schema)

#### Work with Wikidata API
→ See [CONTRIBUTORS.md - Wikidata Integration](./CONTRIBUTORS.md#wikidata-integration)

#### Run tests
→ See [CONTRIBUTORS.md - Testing](./CONTRIBUTORS.md#testing)

#### Set up comprehensive testing
→ Follow [TESTING-STRATEGY.md](./TESTING-STRATEGY.md)

#### See test results and metrics
→ Read [TEST-RESULTS.md](./TEST-RESULTS.md) and [VERIFICATION-REPORT.md](./VERIFICATION-REPORT.md)

---

## Documentation Structure

```
docs/
├── README.md (this file)               # Documentation index
│
├── QUICKSTART.md                       # Quick start guide
├── CLAUDE.md                           # Full specification
├── CONSOLIDATED-ARCHITECTURE.md        # Architecture details
│
├── CONTRIBUTORS.md                     # Developer guide
├── MCP-SERVER-GUIDE.md                # MCP integration
├── CLAUDE-SKILLS-GUIDE.md             # Skills creation
│
├── PHASE3-COMPLETE.md                 # Phase 3 results
├── TEST-RESULTS.md                    # Test verification
├── VERIFICATION-REPORT.md             # Verification details
├── FINAL-SUMMARY.md                   # Project summary
│
├── AGGRESSIVE-MODE-SUMMARY.md         # Aggressive mode
├── CONSOLIDATION-SUMMARY.md           # Consolidation notes
└── AGENTS.md                          # Agent docs
```

---

## Key Concepts

### Wikidata Q16917
Q16917 is a Wikidata identifier (QID) for concepts like "hospital". The system uses QIDs to:
- Map concepts across languages (e.g., "hospital" → "医院" → "hôpital")
- Select the most token-efficient form
- Enable cross-lingual optimization

### Database-Backed Patterns
As of v0.4, all optimization patterns are stored in SQLite:
- **Dynamic updates** via HITL feedback
- **Bayesian confidence** scoring with real user data
- **Hot reload** without restart
- **Pattern statistics** and tracking

### HITL (Human-in-the-Loop)
User feedback improves optimization quality:
- Accept/reject/modify optimizations
- Updates pattern confidence scores
- Bayesian inference for calibration
- 90%+ accept rate on high-confidence patterns

### Protected Regions
Automatically preserves:
- Code blocks (```code```)
- URLs (http://...)
- Identifiers (camelCase, snake_case)
- Templates ({{variable}})
- Special syntax

### Token Savings
- **15-40%** on typical prompts
- **40-60%** on verbose prompts
- **70-85%** on boilerplate-heavy prompts (aggressive mode)
- **Zero semantic loss** with Bayesian confidence

---

## Contributing

We welcome contributions! Please see [CONTRIBUTORS.md](./CONTRIBUTORS.md) for:
- Code of conduct
- Development workflow
- Coding standards
- Testing requirements
- Pull request process

---

## Support

- **Issues**: [GitHub Issues](https://github.com/your-org/prompt-polyglot/issues)
- **Discussions**: [GitHub Discussions](https://github.com/your-org/prompt-polyglot/discussions)
- **Documentation**: This folder
- **API Reference**: [CONSOLIDATED-ARCHITECTURE.md](./CONSOLIDATED-ARCHITECTURE.md)

---

## Version History

- **v1.0** (2024-10): Production release with 62/62 tests passing
- **v0.4** (2024-10): Database-backed patterns with HITL integration
- **v0.3** (2024-10): Aggressive mode (83% savings)
- **v0.2** (2024-10): Protected regions, proper capitalization
- **v0.1** (2024-09): Initial release

---

**Happy optimizing!**
