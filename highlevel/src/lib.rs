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

pub use gaborator_sys::{Coef, CoefMeta, Params as GaboratorParams};


/// Reprepresents C++'s `gaborator::coefs<float>`
/// Can be memory-hungry.
/// 
/// (I'm not sure whether this can be dropped after `Analyzer`.
/// I see some mention of reference counting withing Gaborator library,
/// but have not checked in detail.)
pub struct Coefs(gaborator_sys::cxx::UniquePtr<gaborator_sys::Coefs>);

impl Coefs {
    /// Create new instance of Gaborator analyzer/synthesizer based on supplied parameters
    pub fn new(gab: &Gaborator) -> Self {
        Coefs(
            gaborator_sys::create_coefs(&*gab.0)
        )
    }

    /// Allow the coefficients for points in time before limit 
    /// (a time in units of samples) to be forgotten.
    /// Streaming applications can use this to free memory used by coefficients
    /// that are no longer needed. Coefficients that have been forgotten will read as zero.
    /// This does not guarantee that all coefficients before limit are forgotten, only that
    /// ones for limit or later are not, and that the amount of memory consumed by
    /// any remaining coefficients before limit is bounded.
    pub fn forget_before(&mut self, g:&Gaborator, limit: i64, clean_cut: bool)
    {
        gaborator_sys::forget_before(
            &*g.0,
            self.0.pin_mut(),
            limit,
            clean_cut,
        )
    }

    /// Read or write values within `Coefs`, skipping over non-existent entries.
    /// Corresponds to `process` function of Gaborator.
    /// `from_band` and `to_band` may be given INT_MIN / INT_MAX values, that would mean all bands.
    /// `from_sample_time` and `to_sample_time` can also be given INT64_MIN / INT64_MAX value to mean all available data.
    pub fn process(
        &mut self,
        from_band: i32,
        to_band: i32,
        from_sample_time: i64,
        to_sample_time: i64,
        callback: impl FnMut(CoefMeta, &mut Coef),
    ) {
        gaborator_sys::process(
            self.0.pin_mut(),
            from_band,
            to_band,
            from_sample_time,
            to_sample_time,
            &mut gaborator_sys::ProcessOrFillCallback(Box::new(callback)),
        )
    }

    /// Write values to `Coefs`, creating non-existent entries as needed.
    /// Corresponds to `fill` function of Gaborator.
    /// `from_band` and `to_band` may be given INT_MIN / INT_MAX values, that would mean all bands.
    /// `from_sample_time` / `to_sample_time` should not be set to overtly large range, lest memory will be exhausted.
    pub fn fill(
        &mut self,
        from_band: i32,
        to_band: i32,
        from_sample_time: i64,
        to_sample_time: i64,
        callback: impl FnMut(CoefMeta, &mut Coef),
    ) {
        gaborator_sys::fill(
            self.0.pin_mut(),
            from_band,
            to_band,
            from_sample_time,
            to_sample_time,
            &mut gaborator_sys::ProcessOrFillCallback(Box::new(callback)),
        )
    }
}


/// Main type of the crate. Represents C++'s `gaborator::analyzer<float>`.
pub struct Gaborator(gaborator_sys::cxx::UniquePtr<gaborator_sys::Analyzer>);


impl Gaborator {
    /// Create new instance of Gaborator analyzer/synthesizer based on supplied parameters
    pub fn new(params: &GaboratorParams) -> Self {
        Gaborator(
            gaborator_sys::new_analyzer(params)
        )
    }

    /// Returns the one-sided worst-case time domain support of any of the analysis filters.
    /// When calling `analyze()` with a sample at time t, only spectrogram coefficients within
    /// the time range t ± support will be significantly changed. Coefficients outside the range
    /// may change, but the changes will sufficiently small that they may be ignored without significantly reducing accuracy.
    pub fn analysis_support_len(&self) -> usize { gaborator_sys::get_analysis_support_len(&*self.0) }

    /// Returns the one-sided worst-case time domain support of any of the reconstruction filters.
    /// When calling synthesize() to synthesize a sample at time t, the sample will only be significantly
    /// affected by spectrogram coefficients in the time range t ± support. Coefficients outside the range
    /// may be used in the synthesis, but substituting zeroes for the actual coefficient values will not significantly reduce accuracy.
    pub fn synthesis_support_len(&self) -> usize { gaborator_sys::get_synthesis_support_len(&*self.0) }

    /// Return the smallest valid bandpass band number, corresponding to the highest-frequency bandpass filter.
    /// 
    /// The frequency bands of the analysis filter bank are numbered by nonnegative integers that
    /// increase towards lower (sic) frequencies. There is a number of bandpass bands corresponding
    /// to the logarithmically spaced bandpass analysis filters, from near 0.5 (half the sample rate)
    /// to near fmin, and a single lowpass band containing the residual signal from frequencies below fmin.
    pub fn  bandpass_bands_begin(&self) -> i32 { gaborator_sys::bandpass_bands_begin(&*self.0) }

    /// Return the bandpass band number one past the highest valid bandpass band number,
    /// corresponding to one past the lowest-frequency bandpass filter. 
    pub fn  bandpass_bands_end(&self) -> i32 { gaborator_sys::bandpass_bands_end(&*self.0) }

    /// Return the band number of the lowpass band. 
    pub fn  band_lowpass(&self)  -> i32 { gaborator_sys::band_lowpass(&*self.0) }

    /// Return the band number corresponding to the reference frequency `ff_ref`.
    /// If `ff_ref` falls within the frequency range of the bandpass filter bank, this will be a valid bandpass band number, otherwise it will not. 
    pub fn  band_ref(&self) -> i32 { gaborator_sys::band_ref(&*self.0) }

    /// Return the center frequency of band number `band`, in units of the sampling frequency. 
    pub fn  band_ff(&self, band: i32) -> f64 { gaborator_sys::band_ff(&*self.0, band) }

    /// Spectrum analyze the samples at `signal` and add the resulting coefficients to `coefs`.
    /// `t1` parameter from Gaborator's `analyze` method is caluclated based on supplied slice size.
    ///
    /// If the `coefs` object already contains some coefficients, the new coefficients are summed to those already present.
    pub fn analyze(
        &self,
        signal: &[f32],
        signal_begin_sample_number: i64,
        coefs: &mut Coefs,
    ) {
        gaborator_sys::analyze(
            &*self.0,
            signal,
            signal_begin_sample_number,
            coefs.0.pin_mut(),
        )
    }
        
    /// Synthesize signal samples from the coefficients `coef` and store them at `signal`. 
    /// `t1` parameter from Gaborator's `synthesize` method is caluclated based on supplied slice size.
    /// 
    /// The time range may extend outside the range analyzed using analyze(), in which case
    /// the signal is assumed to be zero in the un-analyzed range.
    pub fn synthesize(
        &self,
        coefs: &Coefs,
        signal_begin_sample_number: i64,
        signal: &mut [f32],
    ) {
        gaborator_sys::synthesize(
            &*self.0,
            &*coefs.0,
            signal_begin_sample_number,
            signal,
        )
    }
}