mod smf;
mod wav;

struct Channel {
    volume: f32,
    pan: f32,
}

struct Note {
    start: f64,
    channel: u32,
    notenum: u32,
    phase: f32,
    d_phase: f32,
    gain: f32,
}

fn main() {
    let events = smf::load("youkoso.mid");
    let mut writer = wav::Writer::new("output.wav");
    let mut event_it = events.iter().peekable();
    let mut channels: Vec<Channel> = (0..events
        .iter()
        .map(|e| match e {
            smf::Event::On { channel, .. } => *channel,
            smf::Event::Off { channel, .. } => *channel,
            smf::Event::Volume { channel, .. } => *channel,
            smf::Event::Pan { channel, .. } => *channel,
            smf::Event::Tempo { .. } => 0,
        })
        .max()
        .unwrap()
        + 1)
        .map(|_| Channel { volume: 100.0 / 127.0, pan: 0.0 })
        .collect();
    let mut notes: Vec<Note> = Vec::new();
    let sample_rate: u32 = 44100;
    let mut tick = 0.0;
    let mut tempo = 120.0;
    'main: for i in 0..sample_rate * 60 * 10 {
        let time = i as f64 / sample_rate as f64;
        let (mut l, mut r) = (0.0, 0.0);
        for note in &mut notes {
            // let s = (note.phase * std::f32::consts::PI * 2.0).sin();
            let s = if note.phase < 0.5 { 1.0 } else { -1.0 };
            let channel = &channels[note.channel as usize];
            let env = 0.8 / (1.0 + (time - note.start) * 4.0) as f32 + 0.2;
            let s = s * channel.volume * note.gain * env * 0.1;
            let (ll, rr) = panning(channel.pan, s);
            l += ll;
            r += rr;
            note.phase += note.d_phase;
            note.phase = note.phase.fract();
        }
        writer.write(l, r);

        tick += tempo as f64 / 60.0 / sample_rate as f64;
        loop {
            let e = if let Some(e) = event_it.peek() {
                e
            } else {
                break 'main;
            };
            if tick < e.tick() {
                break;
            }

            match e {
                smf::Event::On {
                    tick: _,
                    channel,
                    notenum,
                    velocity,
                } => {
                    notes.push(Note {
                        start: time,
                        channel: *channel,
                        notenum: *notenum,
                        phase: 0.0,
                        d_phase: 440.0 * 2.0f32.powf((*notenum as f32 - 69.0) / 12.0)
                            / sample_rate as f32,
                        gain: *velocity * 0.05,
                    });
                }
                smf::Event::Off {
                    tick: _,
                    channel,
                    notenum,
                } => {
                    for i in 0..notes.len() {
                        if notes[i].channel == *channel && notes[i].notenum == *notenum {
                            notes.remove(i);
                            break;
                        }
                    }
                }
                smf::Event::Volume {
                    tick: _,
                    channel,
                    volume,
                } => {
                    channels[*channel as usize].volume = *volume;
                }
                smf::Event::Pan {
                    tick: _,
                    channel,
                    pan,
                } => {
                    channels[*channel as usize].pan = *pan;
                }
                smf::Event::Tempo { tick: _, tempo: t } => {
                    tempo = *t;
                }
            }
            event_it.next();
        }
    }
    writer.finish();
    println!("end ;)");
}

fn panning(pan: f32, input: f32) -> (f32, f32) {
    let pan = pan.min(1.0).max(-1.0);
    let x = (pan + 1.0) / 2.0;
    let gain_l = (x * std::f32::consts::FRAC_PI_2).cos();
    let gain_r = (x * std::f32::consts::FRAC_PI_2).sin();
    (input * gain_l, input * gain_r)
}
