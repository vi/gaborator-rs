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

void read_coefficients(
             int32_t from_band,
             int32_t to_band,
             int64_t from_sample_time,
             int64_t to_sample_time,
             Coefs &coefs,
             rust::Vec<Coef>& output)
{
    gaborator::process(
        [&output](int, int64_t, std::complex<float> &coef) {
            Coef c;
            c.re = real(coef);
            c.im = imag(coef);
            output.push_back(std::move(c));
        },
        (int)from_band,
        (int)to_band,
        from_sample_time,
        to_sample_time,
        coefs);
}

void write_coefficients(
             int32_t from_band,
             int32_t to_band,
             int64_t from_sample_time,
             int64_t to_sample_time,
             Coefs &coefs,
             const rust::Vec<Coef>& input)
{
    rust::Vec<const Coef>::iterator i = input.begin();
    gaborator::fill(
        [&i, &input](int, int64_t, std::complex<float> &coef) {
            if (i != input.end()) {
                coef = std::complex<float>(i->re, i->im);
                ++i;
            } else {
                coef = 0.0;
            }
        },
        (int)from_band,
        (int)to_band,
        from_sample_time,
        to_sample_time,
        coefs);
}

} // namespace gabbridge