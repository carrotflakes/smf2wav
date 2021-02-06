mod smf;
mod wav;
mod context;

fn main() {
    let events = smf::load("youkoso.mid");
    let mut writer = wav::Writer::new("output.wav");
    let sample_rate: usize = 44100;
    let mut context = context::Context::new(sample_rate, events);
    for _ in 0..sample_rate * 60 * 10 {
        if context.is_end() {
            break;
        }
        let (l, r) = context.sample();
        writer.write(l, r);
    }
    writer.finish();
    println!("end ;)");
}
