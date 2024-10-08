Comprehend Drop Check in Rust by Examples

Run `make` to `cargo run` and generate a `profile.svg` that shows the details potential memory
leak. Note that the crate jemallocator seems to have memory leak itself(I don't know why. I'm
not an expert in it.) So don't freak out if you find some function call like
`_rjem_je_prof_backtrace` leaks 64B memory. It is not your fault.

Read these functions and their annotations sequentially to build your mental model about the drop
check system in Rust. You are encouraged to add/delete/modify the code to see the warning/error
from compiler and the output of the program. Playing with it is beneficial to your understand.
The main function is at the end of the file. Scroll down all the way to choose which function to
run.
