fn main() -> anyhow::Result<()> {
    eprintln!("Reading WAV data from stdin");

    let si = std::io::stdin();
    let si = si.lock();
    let si = std::io::BufReader::new(si);
    let mut inp = hound::WavReader::new(si)?;

    if inp.spec().channels != 1 {
        anyhow::bail!("input audio should be mono");
    }
    let sr = inp.spec().sample_rate;
    if !(44000..=49000).contains(&sr) {
        anyhow::bail!("Input sample rate should be around 48000")
    }

    const BUFSIZE : usize = 48000/2;

    let g = gaborator::Gaborator::new(&gaborator::GaboratorParams {
        bands_per_octave: 384,
        ff_min: 200.0 / (sr as f64),
        ff_ref: 440.0 / (sr as f64),
        overlap: 0.7,
    });

    let mut coefs = gaborator::Coefs::new(&g);
    let mut sample_time : i64 = 0;

    let a_s = g.analysis_support_len() as i64;

    // Assuming lowpass band is always the last one
    let mut magnitudes: Vec<f32> = Vec::with_capacity(g.band_lowpass() as usize + 1);
    
    loop {
        let samples : Vec<f32> = if inp.spec().sample_format == hound::SampleFormat::Float {
            inp.samples::<f32>().take(BUFSIZE).collect::<Result<Vec<_>,_>>()?
        } else {
            inp.samples::<i32>().take(BUFSIZE).map(|x|x.map(|s|s as f32 / 32768.00)).collect::<Result<Vec<_>,_>>()?
        };

        if samples.is_empty() { break; }

        g.analyze(&samples, sample_time, &mut coefs);

        magnitudes.clear();
        
        coefs.process(0, 10000000, sample_time, sample_time + BUFSIZE as i64, |m,c| {
            let (magn, mut _phase) = num_complex::Complex::new(c.re, c.im).to_polar();
            let idx : usize = m.band as usize;
            if magnitudes.len() <= idx { magnitudes.resize(idx + 1, 0.0); }
            magnitudes[idx] += magn;
        });

        // no argmax in Rust
        let mut winning_magnitude = 0.0;
        let mut winning_magnitude_idx = -1;

        for (idx, m) in magnitudes.iter().enumerate() {
            if idx == g.band_lowpass() as usize { continue; }
            if winning_magnitude < *m {
                winning_magnitude = *m;
                winning_magnitude_idx = idx as i32;
            }
        }
        
        coefs.process(0, 10000000, sample_time, sample_time + BUFSIZE as i64, |m,c| {
            if m.band == winning_magnitude_idx {
                let (magn, phase) = num_complex::Complex::new(c.re, c.im).to_polar();
                println!("{:9.1} {:5.2} {:8.2} {:8.3}",
                    m.sample_time as f32 / sr as f32,
                    g.band_ff(winning_magnitude_idx) * (sr as f64),
                    magn * 1000.0,
                    phase,
                );
            }
        });

        
        sample_time += BUFSIZE as i64;
        coefs.forget_before(&g, sample_time - a_s, false);
    }
    Ok(())
}