fn main() -> anyhow::Result<()> {
    let sr = 48000;

    let g = gaborator::Gaborator::new(&gaborator::GaboratorParams {
        bands_per_octave: 256,
        ff_min: 200.0 / (sr as f64),
        ff_ref: 440.0 / (sr as f64),
        overlap: 0.7,
    });

    let mut coefs = gaborator::Coefs::new(&g);

    let mut max_sample_number = 0;

    let si = std::io::stdin();
    let si = si.lock();
    let si = std::io::BufReader::new(si);

    use std::io::BufRead;

    let mut database: std::collections::HashMap<gaborator::CoefMeta, gaborator::Coef> = std::collections::HashMap::with_capacity(1024*1024*10);

    for l in si.lines() {
        let l: String = l?;
        let splits: Vec<&str> = l.split(',').collect();
        let sample_time: i64 = splits[0].parse()?;
        let band: i32 = splits[1].parse()?;
        let magnitude: f32 = splits[2].parse()?;
        let phase: f32 = splits[3].parse()?;

        let q = num_complex::Complex::from_polar(magnitude, phase);
        let c = gaborator::Coef { re: q.re, im: q.im };
        let m = gaborator::CoefMeta { band, sample_time };

        database.insert(m, c);

        max_sample_number = max_sample_number.max(sample_time);
    }

    coefs.fill(
        -100000,
        100000,
        0,
        max_sample_number,
        
            |meta,coef| {
                if let Some(c) = database.get(&meta) {
                    *coef = *c;
                } else {
                    *coef = gaborator::Coef::default();
                }
            }
        ,
    );

    let mut samples: Vec<f32> = vec![0.0; max_sample_number as usize + 100];

    g.synthesize(&coefs, 0, &mut samples);

    let mut outp = hound::WavWriter::create(
        "output.wav",
        hound::WavSpec {
            channels: 1,
            sample_rate: sr,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        },
    )?;

    samples
        .into_iter()
        .try_for_each(move |x| -> anyhow::Result<()> {
            outp.write_sample(x)?;
            Ok(())
        })?;

    Ok(())
}
