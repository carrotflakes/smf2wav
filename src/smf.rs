use midly::{MetaMessage, MidiMessage, Smf, TrackEventKind};

#[derive(Debug)]
pub enum Event {
    On {
        time: f64,
        channel: u32,
        notenum: u32,
        velocity: f32,
    },
    Off {
        time: f64,
        channel: u32,
        notenum: u32,
    },
    Pan {
        time: f64,
        channel: u32,
        pan: f32,
    },
}

impl Event {
    pub fn time(&self) -> f64 {
        match self {
            Event::On { time, .. } => *time,
            Event::Off { time, .. } => *time,
            Event::Pan { time, .. } => *time,
        }
    }
}

pub fn load(mid: &str) -> Vec<Event> {
    let data = std::fs::read(mid).unwrap();
    let smf = Smf::parse(&data).unwrap();
    let tpb = match smf.header.timing {
        midly::Timing::Metrical(tpb) => tpb.as_int(),
        midly::Timing::Timecode(_, _) => {
            panic!()
        }
    };
    let mut events = Vec::new();
    let mut tempo = 120.0;
    for track in &smf.tracks {
        let mut time = 0.0;
        for event in track {//dbg!(event.delta.as_int());
            time += event.delta.as_int() as f64 / tpb as f64 * 60.0 / tempo as f64;
            match event.kind {
                TrackEventKind::Midi { channel, message } => match message {
                    MidiMessage::NoteOff { key, vel: _ } => {
                        events.push(Event::Off {
                            time,
                            channel: channel.as_int() as u32,
                            notenum: key.as_int() as u32,
                        });
                    }
                    MidiMessage::NoteOn { key, vel } => {
                        events.push(Event::On {
                            time,
                            channel: channel.as_int() as u32,
                            notenum: key.as_int() as u32,
                            velocity: vel.as_int() as f32 / 127.0,
                        });
                    }
                    MidiMessage::Aftertouch { key: _, vel: _ } => {}
                    MidiMessage::Controller { controller, value } => match controller.as_int() {
                        10 => {
                            events.push(Event::Pan {
                                time,
                                channel: channel.as_int() as u32,
                                pan: ((value.as_int() as f32 - 64.0) / 63.0).max(-1.0),
                            });
                        }
                        _ => {}
                    },
                    MidiMessage::ProgramChange { program: _ } => {}
                    MidiMessage::ChannelAftertouch { vel: _ } => {}
                    MidiMessage::PitchBend { bend: _ } => {}
                },
                TrackEventKind::SysEx(_) => {}
                TrackEventKind::Escape(_) => {}
                TrackEventKind::Meta(m) => match m {
                    MetaMessage::TrackNumber(_) => {}
                    MetaMessage::Text(_) => {}
                    MetaMessage::Copyright(_) => {}
                    MetaMessage::TrackName(_) => {}
                    MetaMessage::InstrumentName(_) => {}
                    MetaMessage::Lyric(_) => {}
                    MetaMessage::Marker(_) => {}
                    MetaMessage::CuePoint(_) => {}
                    MetaMessage::ProgramName(_) => {}
                    MetaMessage::DeviceName(_) => {}
                    MetaMessage::MidiChannel(_) => {}
                    MetaMessage::MidiPort(_) => {}
                    MetaMessage::EndOfTrack => {}
                    MetaMessage::Tempo(t) => {
                        tempo = 60_000_000.0 / t.as_int() as f64;
                        dbg!(tempo);
                    }
                    MetaMessage::SmpteOffset(_) => {}
                    MetaMessage::TimeSignature(_, _, _, _) => {}
                    MetaMessage::KeySignature(_, _) => {}
                    MetaMessage::SequencerSpecific(_) => {}
                    MetaMessage::Unknown(_, _) => {}
                },
            }
        }
    }
    events.sort_by(|a, b| a.time().partial_cmp(&b.time()).unwrap());
    events
}
