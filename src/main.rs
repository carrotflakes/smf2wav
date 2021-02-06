mod smf;
mod wav;

use smf::Event;

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

struct Context {
    sample_rate: usize,
    events: Vec<Event>,
    i: usize,
    channels: Vec<Channel>,
    notes: Vec<Note>,
    time: f64,
    tempo: f64,
    tick: f64,
}

impl Context {
    pub fn new(events: Vec<Event>) -> Self {
        let channel_num = events
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
            + 1;
        let channels: Vec<Channel> = (0..channel_num)
            .map(|_| Channel {
                volume: 100.0 / 127.0,
                pan: 0.0,
            })
            .collect();
        Context {
            sample_rate: 44100,
            events,
            i: 0,
            channels,
            notes: Vec::new(),
            time: 0.0,
            tempo: 120.0,
            tick: 0.0,
        }
    }

    fn proc_event(&mut self) {
        loop {
            let e = if let Some(e) = self.events.get(self.i) {
                e
            } else {
                return;
            };
            if self.tick < e.tick() {
                break;
            }

            match e {
                smf::Event::On {
                    tick: _,
                    channel,
                    notenum,
                    velocity,
                } => {
                    self.notes.push(Note {
                        start: self.time,
                        channel: *channel,
                        notenum: *notenum,
                        phase: 0.0,
                        d_phase: 440.0 * 2.0f32.powf((*notenum as f32 - 69.0) / 12.0)
                            / self.sample_rate as f32,
                        gain: *velocity * 0.05,
                    });
                }
                smf::Event::Off {
                    tick: _,
                    channel,
                    notenum,
                } => {
                    for i in 0..self.notes.len() {
                        if self.notes[i].channel == *channel && self.notes[i].notenum == *notenum {
                            self.notes.remove(i);
                            break;
                        }
                    }
                }
                smf::Event::Volume {
                    tick: _,
                    channel,
                    volume,
                } => {
                    self.channels[*channel as usize].volume = *volume;
                }
                smf::Event::Pan {
                    tick: _,
                    channel,
                    pan,
                } => {
                    self.channels[*channel as usize].pan = *pan;
                }
                smf::Event::Tempo { tick: _, tempo: t } => {
                    self.tempo = *t;
                }
            }
            self.i += 1;
        }
    }

    pub fn sample(&mut self, time: f64) -> (f32, f32) {
        self.time = time;
        self.proc_event();
        let (mut l, mut r) = (0.0, 0.0);
        for note in &mut self.notes {
            // let s = (note.phase * std::f32::consts::PI * 2.0).sin();
            let s = if note.phase < 0.5 { 1.0 } else { -1.0 };
            let channel = &self.channels[note.channel as usize];
            let env = 0.8 / (1.0 + (self.time - note.start) * 4.0) as f32 + 0.2;
            let s = s * channel.volume * note.gain * env * 0.1;
            let (ll, rr) = panning(channel.pan, s);
            l += ll;
            r += rr;
            note.phase += note.d_phase;
            note.phase = note.phase.fract();
        }
        self.tick += self.tempo as f64 / 60.0 / self.sample_rate as f64;
        (l, r)
    }

    pub fn is_end(&self) -> bool {
        self.i == self.events.len()
    }
}

fn main() {
    let events = smf::load("youkoso.mid");
    let mut writer = wav::Writer::new("output.wav");
    let sample_rate: u32 = 44100;
    let mut context = Context::new(events);
    for i in 0..sample_rate * 60 * 10 {
        if context.is_end() {
            break;
        }
        let (l, r) = context.sample(i as f64 / sample_rate as f64);
        writer.write(l, r);
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
