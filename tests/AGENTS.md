## Common
- Never write files outside home directory. Prefer project directory, or use ~/.cache directory if needed, for temporary files.

## Style
- Don't add comments that just describe what simple enough code does when it's obvious from reading it.

## Testing instructions
- Avoid using real service names in tests (such as `nginx`), use `test-service` instead.
- When you add mock/fake output in integration tests, include a comment literally: `// TODO: auto-generated, check or replace.`.
- When editing or adding tests, always do a final check where you run all the tests.
