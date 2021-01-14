fn main() -> anyhow::Result<()> {
    let inp = hound::WavReader::open("input.wav")?;
    if inp.spec().channels != 1 {
        anyhow::bail!("input.wav should be mono");
    }
    let sr = inp.spec().sample_rate;
    if !(44000..=49000).contains(&sr) {
        anyhow::bail!("Input sample rate should be around 48000")
    }
    let samples : Vec<f32> = if inp.spec().sample_format == hound::SampleFormat::Float {
        inp.into_samples::<f32>().collect::<Result<Vec<_>,_>>()?
    } else {
        inp.into_samples::<i32>().map(|x|x.map(|s|s as f32 / 32768.00)).collect::<Result<Vec<_>,_>>()?
    };

    let g = gaborator_sys::new_analyzer(&gaborator_sys::Params {
        bands_per_octave: 24,
        ff_min: 200.0 / (sr as f64),
        ff_ref: 440.0 / (sr as f64),
        overlap: 0.7,
    });

    let mut coefs = gaborator_sys::create_coefs(&g);

    gaborator_sys::analyze(&g, &samples, 0, coefs.pin_mut());


    let num_samples = samples.len();

    let so = std::io::stdout();
    let so = so.lock();
    let mut so = std::io::BufWriter::new(so);

    const BUF_SIZE : usize = 24000;

    for i in 0..(num_samples as usize + 1) / BUF_SIZE {
        use std::io::Write;

        gaborator_sys::process(
            coefs.pin_mut(),
            -100000,
            100000,
            (BUF_SIZE * i) as i64,
            (BUF_SIZE * (i+1)) as i64,
            &mut gaborator_sys::ProcessOrFillCallback(Box::new(
                |meta,coef| {
                    let (magn, mut _phase) = num_complex::Complex::new(coef.re, coef.im).to_polar();
                    let _ = writeln!(so.get_mut(), "{},{},{},{}", meta.sample_time, meta.band, magn, _phase);
                }
            )));
    }
    Ok(())
}