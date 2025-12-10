## Common
- Never write files outside home directory. Prefer project directory, or use ~/.cache directory if needed, for temporary files.

## Style
- Only add comments if the code is complicated and intent is not clear.

## Testing instructions
- Avoid using real service names in tests (such as `nginx`), use `test-service` instead.
- When you add mock/fake output in integration tests, include a comment literally: `// TODO: auto-generated, check or replace.`.
- When editing or adding tests, always do a final check where you run all the tests.
- When you're in a tight spot, you can use println!() to debug struct contents etc.
