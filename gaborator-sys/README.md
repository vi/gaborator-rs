# gaborator-sys

Gaborator is a C++ library for converting audio samples to a special spectral representation
that uses different FTT sizes based on whether it is bass or treble (oversimplifying here).
The transformation is reversible.
See [the website](https://www.gaborator.com/) for more info.

This crate is a [cxx](https://cxx.rs/)-based wrapper of this library, allowing Rust code to use Gaborator (although with reduced efficiency).

Limitations:

* `f32` only
* Not performance-minded
* Some overridable or low-level details not exposed
* No visualisation
* Crate soundness may be iffy - I was just followed the path of least resistance.
* Arithmentic overflows in buffer length calculations are not checked.
* Not really tested, apart from included examples. For example, streaming should be supported, but I haven't tried it myself.

Currently based on Gaborator version 1.6. Source code of the Gaborator is included into the crate.

Availble examples:

* Phase information randomizer, creating sort-of-reverberation audio effect.
* Converts the analyzed sound to (sample,band,magnitude,phase) CSV file and back.

License of Gaborator is Affero GPL 3.0.

Glue code (sans doccomments copied from Gaborator) in this crate may be considered
to be licensed as either MIT or AGPL-3.0, at your option.

License: AGPL-3.0
