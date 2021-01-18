use crate::audio::SoundSource;
use crate::tests;
use crate::*;

#[test]
fn audio_load_ogg() {
    let (c, _e) = &mut tests::make_context();

    // OGG format
    let filename = "/pew.ogg";
    let _sound = audio::Source::new(c, filename).unwrap();
    let _sound = audio::SpatialSource::new(c, filename).unwrap();
}

#[test]
fn audio_load_wav() {
    let (c, _e) = &mut tests::make_context();

    // WAV format
    let filename = "/pew.wav";
    let _sound = audio::Source::new(c, filename).unwrap();
    let _sound = audio::SpatialSource::new(c, filename).unwrap();
}

#[test]
fn audio_load_flac() {
    let (c, _e) = &mut tests::make_context();

    // FLAC format
    let filename = "/pew.flac";
    let _sound = audio::Source::new(c, filename).unwrap();
    let _sound = audio::SpatialSource::new(c, filename).unwrap();
}

#[test]
fn fail_when_loading_nonexistent_file() {
    let (c, _e) = &mut tests::make_context();

    // File does not exist
    let filename = "/does-not-exist.ogg";
    assert!(audio::Source::new(c, filename).is_err());
    assert!(audio::SpatialSource::new(c, filename).is_err());
}

#[test]
fn fail_when_loading_non_audio_file() {
    let (c, _e) = &mut tests::make_context();

    let filename = "/player.png";
    assert!(audio::Source::new(c, filename).is_err());
    assert!(audio::SpatialSource::new(c, filename).is_err());
}

#[test]
fn playing_returns_correct_value_based_on_state() {
    let (c, _e) = &mut tests::make_context();

    let mut sound = audio::Source::new(c, "/pew.ogg").unwrap();
    assert!(!sound.playing());

    sound.play(c).unwrap();
    assert!(sound.playing());

    sound.pause();
    assert!(!sound.playing());

    sound.resume();
    assert!(sound.playing());

    sound.stop(c).unwrap();
    assert!(!sound.playing());
}

#[test]
fn paused_returns_correct_value_based_on_state() {
    let (c, _e) = &mut tests::make_context();

    let mut sound = audio::Source::new(c, "/pew.ogg").unwrap();
    assert!(!sound.paused());

    sound.play(c).unwrap();
    assert!(!sound.paused());

    sound.pause();
    assert!(sound.paused());

    sound.resume();
    assert!(!sound.paused());

    sound.pause();
    assert!(sound.paused());

    sound.stop(c).unwrap();
    assert!(!sound.paused());
}

#[test]
fn volume_persists_after_stop() {
    let (mut c, _e) = tests::make_context();
    let filename = "/pew.ogg";
    let s1 = audio::Source::new(&mut c, filename).unwrap();
    test_volume_after_stop(&mut c, s1);
    let s2 = audio::SpatialSource::new(&mut c, filename).unwrap();
    test_volume_after_stop(&mut c, s2);

    fn test_volume_after_stop(c: &mut Context, mut sound: impl SoundSource) {
        let volume = 0.8;
        sound.set_volume(volume);
        assert_eq!(sound.volume(), volume);
        sound.stop(c).unwrap();
        assert_eq!(sound.volume(), volume);
    }
}

#[test]
fn volume_persists_after_play() {
    let (c, _e) = &mut tests::make_context();
    let filename = "/pew.ogg";
    let s1 = audio::Source::new(c, filename).unwrap();
    test_volume(c, s1);
    let s2 = audio::SpatialSource::new(c, filename).unwrap();
    test_volume(c, s2);

    fn test_volume(c: &mut Context, mut sound: impl SoundSource) {
        let volume = 0.8;
        assert_eq!(sound.volume(), 1.0);
        sound.set_volume(volume);
        assert_eq!(sound.volume(), volume);
        sound.play(c).unwrap();
        assert_eq!(sound.volume(), volume);
    }
}
