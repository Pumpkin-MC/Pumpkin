# 📚 Pumpkin i18n & Text API Documentation Index

> **Last Updated**: 2026-06-30

The documentation in this directory has been reorganized into three specialized files, divided by functional domain:

---

### [🌐 Translation System Complete Workflow](./TRANSLATION_SYSTEM_FLOW.md)

Complete end-to-end flow documentation — from compile-time embedding to runtime query.

- System architecture overview and layered design
- Build phase: build.rs → embed en_us only
- Startup phase: locale resolution → download → inject engine → init background loader
- Player join flow: locale cache → background download → three-tier fallback
- Runtime query hot path: translate → engine.resolve → FST → DashMap cache → Token formatting
- Translation engine internals: FST index, ArcSwap lock-free storage, Token precompilation
- Runtime download system: HTTP download, SHA256 verification, disk cache, deduplication
- Configuration reference and data flow panorama

---

### [📖 API Reference Manual](./API_REFERENCE.md)

All API signatures, call chains, and complete usage examples.

- Import conventions
- Unified translation entry layer: translate_plain / translate_format / localized_log / localized_log_format /
  localized_text
- Function selection decision table + complete call chain diagram
- pumpkin-i18n module APIs: Locale / Server / Client / Store / Engine / Token / Download
- Text component system: TextComponent / TextComponentBase / TextContent / Style / Color / ClickEvent / HoverEvent
- 13 complete usage examples (init → logging → chat → gradients → dynamic injection)

---

### [🏷️ Translation Key Naming Convention](./NAMING_CONVENTION.md)

Naming rules, placeholder formats, and file structure specifications for translation keys.

- Basic naming rules (dot-separated, all lowercase, underscores)
- Hierarchy structure explanation
- Format quick reference (recommended formats for 17 message types)
- 16 namespace details (complete distribution of 1122 keys)
- 8 placeholder formats and argument mapping
- Translation file directory structure and JSON format
- Complete workflow for adding new translations
