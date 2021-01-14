//! Gaborator is a C++ library for converting audio samples to a special spectral representation
//! that uses different FTT sizes based on whether it is bass or treble (oversimplifying here).
//! The transformation is reversible.
//! See [the website](https://www.gaborator.com/) for more info.
//!
//! This crate is a [cxx](https://cxx.rs/)-based wrapper of this library, allowing Rust code to use Gaborator (although with reduced efficiency).
//!
//! Limitations:
//!
//! * `f32` only
//! * Not performance-minded
//! * Some overridable or low-level details not exposed
//! * No visualisation
//! * Crate soundness may be iffy - I was just followed the path of least resistance.
//! * Arithmentic overflows in buffer length calculations are not checked.
//! * No high-level API with methods.
//! * Not really tested, apart from included examples. For example, streaming should be supported, but I haven't tried it myself.
//!
//! Currently based on Gaborator version 1.6. Source code of the Gaborator is included into the crate.
//! 
//! Availble examples:
//! 
//! * Phase information randomizer, creating sort-of-reverberation audio effect.
//! * Converts the analyzed sound to (sample,band,magnitude,phase) CSV file and back.
//!
//! License of Gaborator is Affero GPL 3.0.
//!
//! Glue code (sans doccomments copied from Gaborator) in this crate may be considered
//! to be licensed as either MIT or AGPL-3.0, at your option.

#![deny(missing_docs)]


#[cxx::bridge(namespace = "gabbridge")]
mod ffi {
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

    /// Complex point, representing one coefficient.
    /// Magnitude is loudness at this point, argument is phase.
    #[derive(Copy,Clone,Debug,PartialEq,PartialOrd,Default)]
    pub struct Coef {
        /// Real part of the complex number
        pub re: f32,
        /// Imaginatry part of the complex number
        pub im: f32,
    }

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

        /// Represents C++'s `gaborator::analyzer<float>`. Use `new_analyzer` function to create it.
        pub type Analyzer;

        /// Reprepresents C++'s `gaborator::coefs<float>`. Create it using `create_coefs`.
        /// Can be memory-hungry.
        /// 
        /// (I'm not sure whether this can be dropped after `Analyzer`.
        /// I see some mention of reference counting withing Gaborator library,
        /// but have not checked in detail.)
        pub type Coefs;

        /// Create new instance of Gaborator analyzer/synthesizer based on supplied parameters
        pub fn new_analyzer(params: &Params) -> UniquePtr<Analyzer>;

        /// Returns the one-sided worst-case time domain support of any of the analysis filters.
        /// When calling `analyze()` with a sample at time t, only spectrogram coefficients within
        /// the time range t ± support will be significantly changed. Coefficients outside the range
        /// may change, but the changes will sufficiently small that they may be ignored without significantly reducing accuracy.
        pub fn get_analysis_support_len(b: &Analyzer) -> usize;

        /// Returns the one-sided worst-case time domain support of any of the reconstruction filters.
        /// When calling synthesize() to synthesize a sample at time t, the sample will only be significantly
        /// affected by spectrogram coefficients in the time range t ± support. Coefficients outside the range
        /// may be used in the synthesis, but substituting zeroes for the actual coefficient values will not significantly reduce accuracy.
        pub fn get_synthesis_support_len(b: &Analyzer) -> usize;

        /// Create `Coefs` - the holder of analyzed data. 
        pub fn create_coefs(b: &Analyzer) -> UniquePtr<Coefs>;

        /// Allow the coefficients for points in time before limit 
        /// (a time in units of samples) to be forgotten.
        /// Streaming applications can use this to free memory used by coefficients
        /// that are no longer needed. Coefficients that have been forgotten will read as zero.
        /// This does not guarantee that all coefficients before limit are forgotten, only that
        /// ones for limit or later are not, and that the amount of memory consumed by
        /// any remaining coefficients before limit is bounded.
        pub fn forget_before(b: &Analyzer, c: Pin<&mut Coefs>, limit: i64, clean_cut: bool);


        /// Read or write values within `Coefs`, skipping over non-existent entries.
        /// Corresponds to `process` function of Gaborator.
        /// `from_band` and `to_band` may be given INT_MIN / INT_MAX values, that would mean all bands.
        /// `from_sample_time` and `to_sample_time` can also be given INT64_MIN / INT64_MAX value to mean all available data.
        pub fn process(
            coefs: Pin<&mut Coefs>,
            from_band: i32,
            to_band: i32,
            from_sample_time: i64,
            to_sample_time: i64,
            callback: &mut ProcessOrFillCallback,
        );

        /// Write values to `Coefs`, creating non-existent entries as needed.
        /// Corresponds to `fill` function of Gaborator.
        /// `from_band` and `to_band` may be given INT_MIN / INT_MAX values, that would mean all bands.
        /// `from_sample_time` / `to_sample_time` should not be set to overtly large range, lest memory will be exhausted.
        pub fn fill(
            coefs: Pin<&mut Coefs>,
            from_band: i32,
            to_band: i32,
            from_sample_time: i64,
            to_sample_time: i64,
            callback: &mut ProcessOrFillCallback,
        );

        /// Spectrum analyze the samples at `signal` and add the resulting coefficients to `coefs`.
        /// `t1` parameter from Gaborator's `analyze` method is caluclated based on supplied slice size.
        ///
        /// If the `coefs` object already contains some coefficients, the new coefficients are summed to those already present.
        pub fn analyze(
            b : &Analyzer,
            signal: &[f32],
            signal_begin_sample_number: i64,
            coefs: Pin<&mut Coefs>,
        );
            
        /// Synthesize signal samples from the coefficients `coef` and store them at `signal`. 
        /// `t1` parameter from Gaborator's `synthesize` method is caluclated based on supplied slice size.
        /// 
        /// The time range may extend outside the range analyzed using analyze(), in which case
        /// the signal is assumed to be zero in the un-analyzed range.
        pub fn synthesize(
            b : &Analyzer,
            coefs: &Coefs,
            signal_begin_sample_number: i64,
            signal: &mut [f32],
        );

        /// Return the smallest valid bandpass band number, corresponding to the highest-frequency bandpass filter.
        /// 
        /// The frequency bands of the analysis filter bank are numbered by nonnegative integers that
        /// increase towards lower (sic) frequencies. There is a number of bandpass bands corresponding
        /// to the logarithmically spaced bandpass analysis filters, from near 0.5 (half the sample rate)
        /// to near fmin, and a single lowpass band containing the residual signal from frequencies below fmin.
        pub fn  bandpass_bands_begin(b : &Analyzer) -> i32;

        /// Return the bandpass band number one past the highest valid bandpass band number,
        /// corresponding to one past the lowest-frequency bandpass filter. 
        pub fn  bandpass_bands_end(b : &Analyzer) -> i32;

        /// Return the band number of the lowpass band. 
        pub fn  band_lowpass(b : &Analyzer)  -> i32;

        /// Return the band number corresponding to the reference frequency `ff_ref`.
        /// If `ff_ref` falls within the frequency range of the bandpass filter bank, this will be a valid bandpass band number, otherwise it will not. 
        pub fn  band_ref(b : &Analyzer) -> i32;

        /// Return the center frequency of band number `band`, in units of the sampling frequency. 
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
