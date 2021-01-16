#include "gaborator-sys/src/gabbridge.h"
#include "gaborator-sys/src/lib.rs.h"

namespace gabbridge {

std::unique_ptr<Analyzer> new_analyzer(const Params &params)
{
    gaborator::parameters params_(params.bands_per_octave, params.ff_min, params.ff_ref, params.overlap);
    std::unique_ptr<Analyzer> t = std::unique_ptr<Analyzer>(new Analyzer(params_));
    return t;
}


size_t get_analysis_support_len(const Analyzer& b) { return ceil(b.analysis_support()); }
size_t get_synthesis_support_len(const Analyzer& b) { return ceil(b.synthesis_support()); }

std::unique_ptr<Coefs> create_coefs(const Analyzer& b)
{
    return std::unique_ptr<Coefs>(new Coefs(b));
}

void forget_before(const Analyzer& b, Coefs& c, int64_t limit, bool clean_cut)
{
    gaborator::forget_before(b, c, limit, clean_cut);
}

void process(
             Coefs &coefs,
             int32_t from_band,
             int32_t to_band,
             int64_t from_sample_time,
             int64_t to_sample_time,
             ProcessOrFillCallback& callback)
{
    gaborator::process(
        [&callback](int b, int64_t st, std::complex<float> &coef) {
            Coef c;
            c.re = real(coef);
            c.im = imag(coef);
            CoefMeta m;
            m.band = b;
            m.sample_time = st;

            process_or_write_callback(callback, m, c);

            coef = std::complex<float>(c.re, c.im);
        },
        (int)from_band,
        (int)to_band,
        from_sample_time,
        to_sample_time,
        coefs);
}


void fill(
             Coefs &coefs,
             int32_t from_band,
             int32_t to_band,
             int64_t from_sample_time,
             int64_t to_sample_time,
             ProcessOrFillCallback& callback)
{
    gaborator::fill(
        [&callback](int b, int64_t st, std::complex<float> &coef) {
            Coef c;
            c.re = real(coef);
            c.im = imag(coef);
            CoefMeta m;
            m.band = b;
            m.sample_time = st;

            process_or_write_callback(callback, m, c);

            coef = std::complex<float>(c.re, c.im);
        },
        (int)from_band,
        (int)to_band,
        from_sample_time,
        to_sample_time,
        coefs);
}

void analyze(const Analyzer& b,
        rust::Slice<const float> signal,
        int64_t signal_begin_sample_number,
        Coefs &coefs)
{
    b.analyze(
        signal.data(),
        signal_begin_sample_number,
        signal_begin_sample_number + signal.length(),
        coefs);
}

void synthesize(const Analyzer& b,
        const Coefs &coefs,
        int64_t signal_begin_sample_number,
        rust::Slice<float> signal)
{
    b.synthesize(
        coefs,
        signal_begin_sample_number,
        signal_begin_sample_number + signal.length(),
        signal.data());
}

} // namespace gabbridge