Where's all the data?
=====================
A library with utility binaries for reading (and in the future maybe also
writing?) WAD files compatible with the Doom game engine.

In scope: Support for existing WAD implementations in any of the authentic games
using variants of this engine.

Out of scope: Support for any of the data formats hosted inside a WAD file.

Try it out
----------
Install via Rust toolchain:

    cargo install wad

Run:

    wad-ls doom1.wad
    wad-read doom1.wad endoom | iconv -f CP437 | sed 's/\(.\)./\1/g' | sed 's/\(.\{80\}\)/\1\n/g'
