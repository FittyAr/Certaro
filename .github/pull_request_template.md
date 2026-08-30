## Summary

<!-- What does this PR do? Why? -->

## Checklist

- [ ] `cargo clippy --workspace --all-targets -- -D warnings` passes
- [ ] `pnpm lint` passes
- [ ] `pnpm typecheck` passes
- [ ] `pnpm i18n:check` passes
- [ ] `cargo test --workspace` passes
- [ ] `pnpm test` passes
- [ ] No hardcoded strings in XAML/Vue (use i18n keys)
- [ ] No literal colors (use design tokens)
- [ ] New/changed business logic has unit tests
- [ ] Documentation updated if needed
