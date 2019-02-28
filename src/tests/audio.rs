use crate::audio::SoundSource;
use crate::tests;
use crate::*;

#[test]
fn audio_load_ogg() {
    let (c, _e) = &mut tests::make_context();

    // OGG format
    let _sound = audio::Source::new(c, "/pew.ogg").unwrap();

    // TODO: This is awkward, we should have a way to check whether
    // a file is valid without trying to play it?
    // let mut sound = audio::Source::new(c, "/player.png").unwrap();
    // sound.play().unwrap();
}

#[test]
fn audio_load_mp3() {
    let (c, _e) = &mut tests::make_context();

    // LAME encoded MP3 format
    let _sound = audio::Source::new(c, "/pew.mp3").unwrap();
}

#[test]
fn audio_load_wav() {
    let (c, _e) = &mut tests::make_context();

    // WAV format
    let _sound = audio::Source::new(c, "/pew.wav").unwrap();
}

#[test]
fn audio_load_flac() {
    let (c, _e) = &mut tests::make_context();

    // FLAC format
    let _sound = audio::Source::new(c, "/pew.flac").unwrap();
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
        let mut sound = audio::Source::new(c, "/pew.ogg").unwrap();
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
        let mut sound = audio::Source::new(c, "/pew.ogg").unwrap();
        sound.set_volume(volume);
        assert_eq!(sound.volume(), volume);
        sound.play().unwrap();
        assert_eq!(sound.volume(), volume);
    }
}
