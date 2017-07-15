spin_bitwise_rs
===========

This Rust library implements a multiple-reader single-writer spinlock based on a single atomic construct.

The particularity of this project is to always prioritise writers even if there are readers that
are currently waiting on the primitive.

TODO
==========

Notes
==========

Some of the code examples have been borrowed from (https://github.com/mvdnes/spin-rs)