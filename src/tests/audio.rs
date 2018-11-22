use crate::tests;
use crate::*;

#[test]
fn audio_load() {
    let (c, _e) = &mut tests::make_context();
    {
        // TODO: Test different sound formats
        let mut sound = audio::Source::new(c, "/pew.ogg").unwrap();
        sound.play().unwrap();
    }

    // TODO: This is awkward, we should have a way to check whether
    // a file is valid without trying to play it?
    // let mut sound = audio::Source::new(c, "/player.png").unwrap();
    // sound.play().unwrap();
}
