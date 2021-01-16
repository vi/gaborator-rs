//! Low-level part of `gaborator` crate.
//!
//! Documentation was removed from it, as it mostly dulicate the `gaborator`'s one.
//!
//! There are two examples, but they do the same thing as the ones included in `gaborator` crate.

#[deny(missing_docs)]

pub extern crate cxx;

#[cxx::bridge(namespace = "gabbridge")]
mod ffi {
    #[deny(missing_docs)] // pub-reexported by the high-level crate 
    /// Corresponds to `gaborator::parameters`.
    pub struct Params {
        /// The number of frequency bands per octave.
        /// Values from 6 to 384 (inclusive) are supported.
        /// Values outside this range may not work, or may cause degraded performance.
        pub bands_per_octave: u32,

        /// The lower limit of the analysis frequency range, in units of the sample rate.
        /// The analysis filter bank will extend low enough in frequency that ff_min falls
        /// between the two lowest frequency bandpass filters. Values from 0.001 to 0.13 are supported.
        pub ff_min: f64,

        /// The reference frequency, in units of the sample rate. This allows fine-tuning of
        /// the analysis and synthesis filter banks such that the center frequency of one of
        /// the filters is aligned with ff_ref. If ff_ref falls outside the frequency range of
        /// the bandpass filter bank, this works as if the range were extended to include ff_ref.
        /// Must be positive. A typical value when analyzing music is 440.0 / fs, where fs is the sample rate in Hz. 
        ///
        /// Default value in C++ code is `1.0`.
        pub ff_ref: f64,

        /// A field not documented in Gaborator reference.
        ///
        /// Default value in C++ code is `0.7`.
        pub overlap: f64,
    }

    #[deny(missing_docs)]
    /// Complex point, representing one coefficient.
    /// Magnitude is loudness at this point, argument is phase.
    #[derive(Copy,Clone,Debug,PartialEq,PartialOrd,Default)]
    pub struct Coef {
        /// Real part of the complex number
        pub re: f32,
        /// Imaginatry part of the complex number
        pub im: f32,
    }

    #[deny(missing_docs)]
    /// Additional data for `read_coefficients_with_meta` or `write_coefficients_with_meta`
    #[derive(Copy,Clone,Debug,PartialEq,PartialOrd,Default,Eq,Ord,Hash)]
    pub struct CoefMeta {
        /// The band number of the frequency band the coefficients pertain to.
        /// This may be either a bandpass band or the lowpass band.
        band: i32,

        /// The point in time the coefficients pertain to, in samples
        sample_time: i64,
    }

    extern "Rust" {
        type ProcessOrFillCallback<'a>;

        fn process_or_write_callback(cb: &mut ProcessOrFillCallback, meta: CoefMeta, coef: &mut Coef);
    }

    unsafe extern "C++" {
        include!("gaborator-sys/src/gabbridge.h");

        pub type Analyzer;
        pub type Coefs;

        pub fn new_analyzer(params: &Params) -> UniquePtr<Analyzer>;

        pub fn get_analysis_support_len(b: &Analyzer) -> usize;
        pub fn get_synthesis_support_len(b: &Analyzer) -> usize;

        pub fn create_coefs(b: &Analyzer) -> UniquePtr<Coefs>;

        pub fn forget_before(b: &Analyzer, c: Pin<&mut Coefs>, limit: i64, clean_cut: bool);


        pub fn process(
            coefs: Pin<&mut Coefs>,
            from_band: i32,
            to_band: i32,
            from_sample_time: i64,
            to_sample_time: i64,
            callback: &mut ProcessOrFillCallback,
        );

        pub fn fill(
            coefs: Pin<&mut Coefs>,
            from_band: i32,
            to_band: i32,
            from_sample_time: i64,
            to_sample_time: i64,
            callback: &mut ProcessOrFillCallback,
        );

        pub fn analyze(
            b : &Analyzer,
            signal: &[f32],
            signal_begin_sample_number: i64,
            coefs: Pin<&mut Coefs>,
        );
            
        pub fn synthesize(
            b : &Analyzer,
            coefs: &Coefs,
            signal_begin_sample_number: i64,
            signal: &mut [f32],
        );

        pub fn  bandpass_bands_begin(b : &Analyzer) -> i32;

        pub fn  bandpass_bands_end(b : &Analyzer) -> i32;

        pub fn  band_lowpass(b : &Analyzer)  -> i32;

        pub fn  band_ref(b : &Analyzer) -> i32;

        pub fn  band_ff(b : &Analyzer, band: i32) -> f64;
    }
}


pub use ffi::*;
/// Wrapper for your callback function for `fill` or `process`.
///
/// Example:
/// 
/// ```no_build
/// gaborator_sys::process(coefs.pin_mut(), -100000, 100000, -100000, 100000, &mut gaborator_sys::ProcessOrFillCallback(Box::new(
///        |_meta,_coef| {
///            // read _meta, read or write _coef
///        }
///    ))); 
/// ```
pub struct ProcessOrFillCallback<'a>(pub Box<dyn FnMut(CoefMeta, &mut Coef) + 'a>);

fn process_or_write_callback(cb: &mut ProcessOrFillCallback, meta: CoefMeta, coef: &mut Coef) {
    cb.0(meta, coef);
}
