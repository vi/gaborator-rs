#pragma once
#include <memory>
#include "gaborator-sys/gaborator/gaborator.h"
#include "rust/cxx.h"

namespace gabbridge {

struct Params;
struct Coef;
enum class WriteCoefficientsMode: uint8_t;

typedef gaborator::analyzer<float> Analyzer;
typedef gaborator::coefs<float> Coefs;

std::unique_ptr<Analyzer> new_analyzer(const Params &params);
size_t get_analysis_support_len(const Analyzer& b);
size_t get_synthesis_support_len(const Analyzer& b);

std::unique_ptr<Coefs> create_coefs(const Analyzer& b);
void forget_before(const Analyzer& b, Coefs& c, int64_t limit, bool clean_cut);

void read_coefficients(
             int32_t from_band,
             int32_t to_band,
             int64_t from_sample_time,
             int64_t to_sample_time,
             Coefs &coefs,
             rust::Vec<Coef>& output);

void write_coefficients(
             int32_t from_band,
             int32_t to_band,
             int64_t from_sample_time,
             int64_t to_sample_time,
             Coefs &coefs,
             const rust::Vec<Coef>& input,
             WriteCoefficientsMode mode);

void analyze(const Analyzer& b,
        rust::Slice<const float> signal,
        int64_t signal_begin_sample_number,
        Coefs &coefs);

void synthesize(const Analyzer& b,
        const Coefs &coefs,
        int64_t signal_begin_sample_number,
        rust::Slice<float> signal);

int32_t bandpass_bands_begin(const Analyzer& b) { return b.bandpass_bands_begin(); }
int32_t bandpass_bands_end(const Analyzer& b) { return b.bandpass_bands_end(); }

int32_t band_lowpass(const Analyzer& b) { return b.band_lowpass(); }
int32_t band_ref(const Analyzer& b) { return b.band_ref(); }

double band_ff(const Analyzer& b, int32_t gbno) { return b.band_ff((int)gbno); }

} // namespace gabbridge