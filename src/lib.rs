//!  Based on `streaming.cc` example from Gaborator source code
//!
//! Limitations:
//! * `f32` only
//! * Not performance-minded
//! * Some overridable or low-level details not exposed
//! * No visualisation
//! * Coefficients must be copied to/from Rust side. `process` need to be called twice - to read and to write.
//! * Crate soundness may be iffy - I was just following the path of least resistance.
//! * Arithmentic overflows in buffer length calculations are not checked.
//!
//! Currently based on Gaborator version 1.6. Source code of Gaborator is included into the crate.
//!
//! License of Gaborator is Affero GPL 3.0.
//!
//! Glue code (sans doccomments copied from Gaborator) in this crate may be considered
//! to be licensed as either MIT or AGPL-3.0, at your option.

#![allow(unused)]



#[cxx::bridge(namespace = "gabbridge")]
pub mod ffi {
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
    pub struct Coef {
        /// Real part of the complex number
        pub re: f32,
        /// Imaginatry part of the complex number
        pub im: f32,
    }

    /// Whether to use `fill` or `process` function of `gaborator`.
    #[repr(u8)]
    pub enum WriteCoefficientsMode {
        /// Use `fill` function, create new coefficients when they are missing from `Coefs`.
        Fill,
        /// Use `process` function, skip non-existing coefficients
        OnlyOverwrite,
    }

    unsafe extern "C++" {
        include!("gaborator-sys/src/gabbridge.h");

        pub type Analyzer;
        pub type Coefs;

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

        /// (I'm not sure whether this can be dropped before GabBridge
        /// I see some mentioned of reference counting withing Gaborator library,
        /// but have not checked in detail.)
        pub fn create_coefs(b: &Analyzer) -> UniquePtr<Coefs>;

        /// Allow the coefficients for points in time before limit 
        /// (a time in units of samples) to be forgotten.
        /// Streaming applications can use this to free memory used by coefficients
        /// that are no longer needed. Coefficients that have been forgotten will read as zero.
        /// This does not guarantee that all coefficients before limit are forgotten, only that
        /// ones for limit or later are not, and that the amount of memory consumed by
        /// any remaining coefficients before limit is bounded.
        pub fn forget_before(b: &Analyzer, c: Pin<&mut Coefs>, limit: i64, clean_cut: bool);

        /// Corresponds to `process` function of Gaborator, sans ability to edit coefficients.
        /// `from_band` and `to_band` may be given INT_MIN / INT_MAX values, that would mean all bands.
        /// `from_sample_time` and `to_sample_time` can also be given INT64_MIN / INT64_MAX value to mean all available data.
        /// Function applied to `process` is a fixed one: it ignores band and time parameter and just 
        /// appends coefficient value to the given vector.
        pub fn read_coefficients(
            from_band: i32,
            to_band: i32,
            from_sample_time: i64,
            to_sample_time: i64,
            coefs: Pin<&mut Coefs>,
            output: &mut Vec<Coef>,
        );

        /// Corresponds to `fill` or `process` function of Gaborator (depending on `mode` parameter)
        /// `from_band` and `to_band` may be given INT_MIN / INT_MAX values, that would mean all bands.
        /// Unlike `read_coefficients`, `from_sample_time` / `to_sample_time` should not be set to overtly large range, lest memory will be exhausted.
        /// Function applied to `process` is a fixed one: it ignores band and time parameter and just 
        /// sets coefficient value based on the given vector.
        /// If vector is too short, remaining coefficients are filled in zeroes.
        pub fn write_coefficients(
            from_band: i32,
            to_band: i32,
            from_sample_time: i64,
            to_sample_time: i64,
            coefs: Pin<&mut Coefs>,
            input: &Vec<Coef>,
            mode: WriteCoefficientsMode,
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