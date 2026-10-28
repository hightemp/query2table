//! Subtitles of a video: which track is read, how its files are read, and the text the
//! research agent gets, with time marks, chapters and pages.

use std::path::{Path, PathBuf};

use query2table_lib::providers::video::subtitles::{self, choose_track, clock, parse, parse_clock, time_link, Cue, TrackKind, Transcript};
use query2table_lib::providers::video::{Chapter, VideoInfo};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/video").join(name)
}

fn read(name: &str) -> String {
    std::fs::read_to_string(fixture(name)).unwrap()
}

fn info(name: &str) -> VideoInfo {
    VideoInfo::from_json(&serde_json::from_str(&read(name)).unwrap()).unwrap()
}

fn cue(start_ms: u64, text: &str) -> Cue {
    Cue { start_ms, text: text.into() }
}

// --- Choosing the track ---

#[test]
fn people_made_subtitles_in_the_videos_language_come_first() {
    let youtube = info("youtube.info.json");
    let track = choose_track(&youtube, None).unwrap();
    assert_eq!((track.lang.as_str(), track.automatic), ("en", false));
    assert_eq!(subtitles::track_kind(&youtube, track), TrackKind::Manual);

    // A named track counts as its language.
    let chapters = info("youtube_chapters.info.json");
    assert_eq!(choose_track(&chapters, None).unwrap().lang, "en-IU4AG2iTnVA");
}

#[test]
fn without_people_made_subtitles_speech_recognition_is_read() {
    let mut youtube = info("youtube_chapters.info.json");
    youtube.tracks.retain(|t| t.automatic);
    let track = choose_track(&youtube, None).unwrap();
    assert_eq!(track.lang, "en-orig");
    assert_eq!(subtitles::track_kind(&youtube, track), TrackKind::Automatic);
}

#[test]
fn another_language_can_be_asked_for() {
    let youtube = info("youtube.info.json");
    // People made a German one: it is preferred to a machine translation.
    let german = choose_track(&youtube, Some("de")).unwrap();
    assert_eq!((german.lang.as_str(), german.automatic), ("de-DE", false));
    // Only YouTube's machine translation exists in Russian.
    let russian = choose_track(&youtube, Some("ru")).unwrap();
    assert_eq!((russian.lang.as_str(), russian.automatic), ("ru", true));
    assert_eq!(subtitles::track_kind(&youtube, russian), TrackKind::Translated);
    assert!(choose_track(&youtube, Some("xx")).is_none());

    let ted = info("ted.info.json");
    assert_eq!(choose_track(&ted, Some("pt")).unwrap().lang, "pt");
    assert_eq!(choose_track(&ted, Some("pt-BR")).unwrap().lang, "pt-br");
    assert_eq!(choose_track(&ted, Some("zh")).unwrap().lang, "zh-cn");
}

#[test]
fn a_video_without_subtitles_has_no_track() {
    let mut ted = info("ted.info.json");
    ted.tracks.clear();
    assert!(choose_track(&ted, None).is_none());
}

#[test]
fn the_best_file_format_is_asked_for() {
    let youtube = info("youtube.info.json");
    assert_eq!(subtitles::format_choice(choose_track(&youtube, None).unwrap()), "json3/vtt/srt/best");
}

// --- Reading subtitle files ---

#[test]
fn youtube_json3_by_people() {
    let cues = parse("json3", &read("youtube.en.json3"));
    assert_eq!(cues[0], cue(1360, "[♪♪♪]"));
    assert_eq!(cues[1], cue(18640, "♪ We're no strangers to love ♪"));
    // Line breaks inside a cue become spaces.
    assert_eq!(cues[2], cue(22640, "♪ You know the rules and so do I ♪"));
}

#[test]
fn youtube_json3_by_speech_recognition() {
    let cues = parse("json3", &read("youtube.en-orig.json3"));
    assert_eq!(cues[0], cue(320, "[Music]"));
    assert_eq!(cues[1], cue(18800, "We're no strangers to"));
    let text = cues.iter().map(|c| c.text.as_str()).collect::<Vec<_>>().join(" ").to_lowercase();
    assert!(text.contains("never going to give you up"), "{text}");
    assert!(cues.iter().all(|c| !c.text.trim().is_empty() && !c.text.contains('\n')));
    assert!(cues.windows(2).all(|w| w[0].start_ms <= w[1].start_ms));
}

#[test]
fn vtt_from_another_site() {
    let cues = parse("vtt", &read("ted.en.vtt"));
    assert_eq!(cues[0], cue(2103, "Good morning. How are you?"));
    assert_eq!(cues[1], cue(4702, "(Audience) Good."));
    assert!(cues.iter().all(|c| !c.text.contains("-->") && !c.text.contains("WEBVTT")));
}

/// YouTube's automatic vtt rolls: every cue repeats the line before it, with word timing tags.
#[test]
fn rolling_vtt_lines_are_read_once() {
    let vtt = "WEBVTT\nKind: captions\nLanguage: en\n\n\
        00:00:01.000 --> 00:00:03.000 align:start position:0%\n\
        hello<00:00:01.500><c> there</c>\n\n\
        00:00:03.000 --> 00:00:03.010 align:start position:0%\n\
        hello there\n \n\n\
        00:00:03.010 --> 00:00:05.000 align:start position:0%\n\
        hello there\n\
        how<00:00:03.500><c> are</c><00:00:04.000><c> you</c>\n\n\
        00:00:05.000 --> 00:00:07.000\n\
        how are you\n\
        fine &amp; you?\n";
    assert_eq!(parse("vtt", vtt), [cue(1000, "hello there"), cue(3010, "how are you"), cue(5000, "fine & you?")]);
}

#[test]
fn srt_files() {
    let srt = "1\r\n00:00:01,500 --> 00:00:03,000\r\n<i>First</i> line\r\nsecond line\r\n\r\n2\r\n01:02:03,000 --> 01:02:04,000\r\nLater\r\n";
    assert_eq!(parse("srt", srt), [cue(1500, "First line second line"), cue(3_723_000, "Later")]);
}

// --- Times and links ---

#[test]
fn clock_times() {
    assert_eq!(clock(0), "0:00");
    assert_eq!(clock(754), "12:34");
    assert_eq!(clock(3723), "1:02:03");
    assert_eq!(parse_clock("40:00"), Some(2400));
    assert_eq!(parse_clock("1:02:03"), Some(3723));
    assert_eq!(parse_clock("754"), Some(754));
    assert_eq!(parse_clock(" 12:34 "), Some(754));
    assert_eq!(parse_clock("12:3x"), None);
}

#[test]
fn links_to_a_moment() {
    assert_eq!(time_link("https://www.youtube.com/watch?v=dQw4w9WgXcQ", 754), "https://www.youtube.com/watch?v=dQw4w9WgXcQ&t=754s");
    assert_eq!(time_link("https://www.youtube.com/watch?v=dQw4w9WgXcQ&t=10s", 754), "https://www.youtube.com/watch?v=dQw4w9WgXcQ&t=754s");
    assert_eq!(time_link("https://youtu.be/dQw4w9WgXcQ", 754), "https://youtu.be/dQw4w9WgXcQ?t=754s");
    assert_eq!(time_link("https://vimeo.com/76979871", 754), "https://vimeo.com/76979871#t=754s");
    assert_eq!(time_link("https://www.ted.com/talks/x", 754), "https://www.ted.com/talks/x#t=754");
    assert_eq!(time_link("https://www.youtube.com/watch?v=x", 0), "https://www.youtube.com/watch?v=x");
}

// --- The text the agent reads ---

fn long_talk() -> Vec<Cue> {
    // Ten minutes, a sentence every five seconds.
    (0..120).map(|i| cue(i * 5000, &format!("Sentence number {i} of the talk."))).collect()
}

#[test]
fn paragraphs_carry_time_marks() {
    let transcript = Transcript::new(&long_talk(), &[]);
    let full = transcript.full(usize::MAX);
    assert!(full.text.starts_with("[0:00] Sentence number 0 of the talk."), "{}", full.text);
    let marks: Vec<&str> = full.text.lines().filter(|l| l.starts_with('[')).map(|l| &l[..l.find(']').unwrap() + 1]).collect();
    assert!(marks.len() >= 5 && marks.len() <= 20, "{marks:?}");
    assert!(full.text.contains("Sentence number 119 of the talk."));
    assert_eq!(full.next, None);
}

#[test]
fn chapters_become_headings() {
    let chapters = [
        Chapter { start: 0, end: 300, title: "Introduction".into() },
        Chapter { start: 300, end: 600, title: "The main part".into() },
    ];
    let text = Transcript::new(&long_talk(), &chapters).full(usize::MAX).text;
    let intro = text.find("## Introduction").unwrap();
    let main = text.find("## The main part").unwrap();
    assert!(intro < text.find("Sentence number 0 ").unwrap());
    assert!(text.find("Sentence number 59 ").unwrap() < main && main < text.find("Sentence number 60 ").unwrap());
    // A paragraph never spans two chapters.
    assert!(text[main..].lines().nth(1).unwrap().starts_with("[5:00]"), "{}", &text[main..]);
}

#[test]
fn a_long_video_is_read_in_pages() {
    let transcript = Transcript::new(&long_talk(), &[]);
    let first = transcript.page(0, 1500);
    assert!(first.text.len() <= 1500);
    assert!(first.text.contains("Sentence number 0 "));
    let next = first.next.expect("more text follows");
    assert!(!first.text.contains(&format!("[{}]", clock(next))));

    let second = transcript.page(next, 1500);
    assert!(second.text.starts_with(&format!("[{}]", clock(next))), "{}", second.text);
    // Every sentence is on exactly one page.
    let mut pages = vec![first.text, second.text];
    let mut from = second.next;
    while let Some(at) = from {
        let page = transcript.page(at, 1500);
        from = page.next;
        pages.push(page.text);
    }
    let all = pages.concat();
    for i in 0..120 {
        assert_eq!(all.matches(&format!("Sentence number {i} ")).count(), 1, "sentence {i}");
    }

    // A time inside a paragraph starts at that paragraph.
    let middle = transcript.page(301, 1500);
    let starts = parse_clock(&middle.text[1..middle.text.find(']').unwrap()]).unwrap();
    assert!(starts <= 301 && starts > 240, "{}", middle.text);
}

#[test]
fn the_whole_text_is_cut_to_the_limit() {
    let transcript = Transcript::new(&long_talk(), &[]);
    let cut = transcript.full(2000);
    assert!(cut.text.len() <= 2000);
    assert!(cut.next.is_some());
    assert!(transcript.page(10_000, 1500).text.is_empty());
}

#[test]
fn empty_cues_and_repeats_are_dropped() {
    let cues = [cue(0, "Hello."), cue(1000, "Hello."), cue(2000, "   "), cue(3000, "World.")];
    let text = Transcript::new(&cues, &[]).full(usize::MAX).text;
    assert_eq!(text, "[0:00] Hello. World.");
}
