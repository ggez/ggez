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

#[test]
fn volume_persists_after_stop() {
    let (c, _e) = &mut tests::make_context();
    {
        let volume = 0.8;
        let mut sound = audio::Source::new(c, "/pew.ogg").unwrap();
        sound.set_volume(volume);
        assert_eq!(sound.volume(), volume);
        sound.stop();
        assert_eq!(sound.volume(), volume);
    }
}

#[test]
fn volume_persists_after_stop_for_spatial_source() {
    let (c, _e) = &mut tests::make_context();
    {
        let volume = 0.8;
        let mut sound = audio::Source::new_spatial(c, "/pew.ogg").unwrap();
        sound.set_volume(volume);
        assert_eq!(sound.volume(), volume);
        sound.stop();
        assert_eq!(sound.volume(), volume);
    }
}

#[test]
fn volume_persists_after_play() {
    let (c, _e) = &mut tests::make_context();
    {
        let volume = 0.8;
        let mut sound = audio::Source::new(c, "/pew.ogg").unwrap();
        sound.set_volume(volume);
        assert_eq!(sound.volume(), volume);
        sound.play().unwrap();
        assert_eq!(sound.volume(), volume);
    }
}

#[test]
fn volume_persists_after_play_for_spatial_source() {
    let (c, _e) = &mut tests::make_context();
    {
        let volume = 0.8;
        let mut sound = audio::Source::new_spatial(c, "/pew.ogg").unwrap();
        sound.set_volume(volume);
        assert_eq!(sound.volume(), volume);
        sound.play().unwrap();
        assert_eq!(sound.volume(), volume);
    }
}
