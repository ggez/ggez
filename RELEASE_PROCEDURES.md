# Things that maintainers should do prior to every release!

 * Note that updating a dep to a breaking version (i.e., nalgebra 0.13 -> 0.14) is a BREAKING
API CHANGE and should not be done on things that don't break API.  For instance, we screwed this
up 'cause ggez 0.4.0 used nalgebra 0.13 and 0.4.1 used nalgebra 0.14... so this broke the exposed
API.  (Doing this for packages that aren't publically exposed is PROBABLY okay...)
 * Fix all rustc warnings
 * Make sure all unit tests pass
 * Test all examples
 * Read all docs (ideally in rendered form)
 * Make sure website is updated and in-sync
 * rustfmt
 * clippy
 * Search for and remove all `expect()` and `unwrap()` calls
 * Search for and address all `TODO` and `BUGGO` comments
 * Make sure readme is updated (should be the same as the top-level crate docs)
 * Make sure changelog is up to date, ideally including full links to issues or commits (not just github issue numbers)
