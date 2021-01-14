fn main() -> anyhow::Result<()> {
    let inp = hound::WavReader::open("input.wav")?;
    if inp.spec().channels != 1 {
        anyhow::bail!("input.wav should be mono");
    }
    let sr = inp.spec().sample_rate;
    if !(44000..=49000).contains(&sr) {
        anyhow::bail!("Input sample rate should be around 48000")
    }
    let mut samples : Vec<f32> = if inp.spec().sample_format == hound::SampleFormat::Float {
        inp.into_samples::<f32>().collect::<Result<Vec<_>,_>>()?
    } else {
        inp.into_samples::<i32>().map(|x|x.map(|s|s as f32 / 32768.00)).collect::<Result<Vec<_>,_>>()?
    };

    let g = gaborator_sys::new_analyzer(&gaborator_sys::Params {
        bands_per_octave: 256,
        ff_min: 200.0 / (sr as f64),
        ff_ref: 440.0 / (sr as f64),
        overlap: 0.7,
    });

    let mut coefs = gaborator_sys::create_coefs(&g);

    gaborator_sys::analyze(&g, &samples, 0, coefs.pin_mut());

    let mut coefs2 : Vec<gaborator_sys::Coef> = Vec::with_capacity(samples.len());

    gaborator_sys::read_coefficients(-100000, 100000, -100000, 10000000000, coefs.pin_mut(), &mut coefs2);

    for gaborator_sys::Coef{ref mut re,ref mut im} in &mut coefs2 {
        let (magn, mut _phase) = num_complex::Complex::new(*re, *im).to_polar();
        _phase *= 100000.0;
        let q = num_complex::Complex::from_polar(magn, _phase);
        *re = q.re;
        *im = q.im;
    }

    gaborator_sys::write_coefficients(-100000, 100000, -100000, 10000000000, coefs.pin_mut(), &mut coefs2, gaborator_sys::WriteCoefficientsMode::OnlyOverwrite);

    gaborator_sys::synthesize(&g, &coefs, 0, &mut samples);

    let mut outp = hound::WavWriter::create("output.wav", hound::WavSpec {
        channels: 1,
        sample_rate: sr,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    })?;

    samples.into_iter().try_for_each(move |x| -> anyhow::Result<()> {outp.write_sample(x)?; Ok(()) } )?;

    Ok(())
}