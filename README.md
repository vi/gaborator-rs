# gaborator-sys

Gaborator is a C++ library for converting audio samples to a special spectral representation
that uses different FTT sizes based on whether it is bass or treble (oversimplifying here).
See [the website](https://www.gaborator.com/) for more info.

This crate is a [cxx](https://cxx.rs/)-based wrapper of this library, allowing Rust code to use Gaborator (although with reduced efficiency).

Limitations:

* `f32` only
* Not performance-minded
* Some overridable or low-level details not exposed
* No visualisation
* Coefficient content must be copied to/from Rust side. `process` needs to be called twice - to read and to write.
* Crate soundness may be iffy - I was just followed the path of least resistance.
* Arithmentic overflows in buffer length calculations are not checked.
* No high-level API with methods.
* Not really tested, apart from included examples.

Currently based on Gaborator version 1.6. Source code of the Gaborator is included into the crate.

There is one example available that randomizes phase information of each coefficient, creating sort-of-reverberation audio effect.

License of Gaborator is Affero GPL 3.0.

Glue code (sans doccomments copied from Gaborator) in this crate may be considered
to be licensed as either MIT or AGPL-3.0, at your option.

License: AGPL-3.0
