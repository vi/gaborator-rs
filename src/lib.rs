//!  Based on `streaming.cc` example from Gaborator source code
//!
//! Limitations:
//! * `f32` only
//! * Not performance-minded
//! * Some overridable or low-level details not exposed
//! * No visualisation
//! * Coefficients must be copied to/from Rust side. `process` need to be called twice - to read and to write.
//! * Crate soundness may be iffy - I was just following the path of least resistance.
//!
//! Currently based on Gaborator version 1.6
//!
//! License of Gaborator is Affero GPL 3.0.
//!
//! Glue code (sans doccomments copied from Gaborator) in this crate may be considered
//! to be licensed as either MIT or AGPL-3.0, at your option.

#![allow(unused)]



#[cxx::bridge(namespace = "gabbridge")]
pub mod ffi {
    struct Params {
        bands_per_octave: u32,
        ff_min: f64,
        ff_ref: f64,
        overlap: f64,
    }
    struct Coef {
        re: f32,
        im: f32,
    }

    unsafe extern "C++" {
        include!("gaborator/src/gabbridge.h");

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

        /// Corresponds to `fill` function of Gaborator.
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
        );
    }
}

pub use ffi::*;