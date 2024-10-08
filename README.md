# Comprehend Drop Check in Rust by Examples

## Setup

There are some unstable features used in the code. Run `rustup default nightly`
to make them available. `jeprof`, which is part of
[jemalloc](https://github.com/jemalloc/jemalloc), would be used to visualize the
memory leak. Installing it is as simple as `conda install conda-forge::jemalloc`
even when you don't a root privilege. If you have, it is the turn of your system
package manager.

## How to Play

Run `make` to `cargo run` and generate a `profile.svg` that shows the details
potential memory leak. Note that the crate jemallocator seems to have memory
leak itself(I don't know why. I'm not an expert in it.) So don't freak out if
you find some function call like `_rjem_je_prof_backtrace` leaks 64B memory. It
is not your fault.

Read these functions and their annotations sequentially to build your mental
model about the drop check system in Rust. You are encouraged to
add/delete/modify the code to see the warning/error from compiler and the output
of the program. Playing with it is beneficial to your understand. The main
function is at the end of the file. Scroll down all the way to choose which
function to run.
