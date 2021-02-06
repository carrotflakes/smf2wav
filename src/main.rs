// use std::fmt::write;

mod smf;
mod wav;

struct Channel {
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
    let mut channels: Vec<Channel> = (0..events.iter().map(|e| match e {
        smf::Event::On { channel, .. } => {*channel}
        smf::Event::Off { channel, .. } => {*channel}
        smf::Event::Pan { channel, .. } => {*channel}
    }).max().unwrap() + 1).map(|_| Channel { pan: 0.0 }).collect();
    let mut notes: Vec<Note> = Vec::new();
    let sample_rate: u32 = 44100;
    'main: for i in 0..sample_rate * 60 * 10 {
        let (mut l, mut r) = (0.0, 0.0);
        for note in &mut notes {
            // sample += (note.phase * std::f32::consts::PI * 2.0).sin() * note.gain;
            let s = if note.phase < 0.5 {1.0} else {-1.0} * note.gain;
            let (ll, rr) = panning(channels[note.channel as usize].pan, s);
            l += ll;
            r += rr;
            note.phase += note.d_phase;
            note.phase = note.phase.fract();
        }
        writer.write(l, r);

        let time = i as f64 / sample_rate as f64;
        loop {
            let e = if let Some(e) = event_it.peek() {
                e
            } else {
                break 'main;
            };
            let etime = e.time() as f64 / 960.0;
            if time < etime {
                break;
            }

            match e {
                smf::Event::On { time: _, channel, notenum, velocity } => {
                    notes.push(Note {
                        start: etime,
                        channel: *channel,
                        notenum: *notenum,
                        phase: 0.0,
                        d_phase: 440.0 * 2.0f32.powf((*notenum as f32 - 69.0) / 12.0) / sample_rate as f32,
                        gain: *velocity * 0.05,
                    });
                }
                smf::Event::Off { time: _, channel, notenum } => {
                    for i in 0..notes.len() {
                        if notes[i].channel == *channel && notes[i].notenum == *notenum {
                            notes.remove(i);
                            break;
                        }
                    }
                }
                smf::Event::Pan { time: _, channel, pan } => {
                    channels[*channel as usize].pan = *pan;
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
