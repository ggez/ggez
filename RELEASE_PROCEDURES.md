# Things that maintainers should do prior to every release!

 * Fix all rustc warnings
 * Make sure all unit tests pass
 * Test all examples
 * Read all docs (ideally in rendered form)
 * Make sure website is updated and in-sync
 * rustfmt
 * clippy
 * Ensure deps are up to date with `cargo outdated`
 * Search for and remove all `expect()` and `unwrap()` calls
 * Search for and address all `TODO` and `BUGGO` comments
 * Make sure readme is updated (should be the same as the top-level crate docs)
