// Comprehend Drop Check in Rust by Examples
//
// Run `make` to `cargo run` and generate a `profile.svg` that shows the details potential memory
// leak. Note that the crate jemallocator seems to have memory leak itself(I don't know why. I'm
// not an expert in it.) So don't freak out if you find some function call like
// `_rjem_je_prof_backtrace` leaks 64B memory. It is not your fault.
//
// Read these functions and their annotations sequentially to build your mental model about the drop
// check system in Rust. You are encouraged to add/delete/modify the code to see the warning/error
// from compiler and the output of the program. Playing with it is beneficial to your understand.
// The main function is at the end of the file. Scroll down all the way to choose which function to
// run.

#![feature(dropck_eyepatch)]

use std::{
    alloc::{self, Layout},
    fmt::Debug,
    marker::PhantomData,
    ptr,
};
use jemallocator::Jemalloc;
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

// drop order: declared first, drop last
#[allow(unused)]
fn drop_order() {
    struct A();
    struct B();
    impl Drop for A {
        fn drop(&mut self) {
            println!("A is dropped last because its declaration is the first")
        }
    }
    impl Drop for B {
        fn drop(&mut self) {
            println!("B is dropped first although its initialization is earlier than A");
        }
    }
    let a;
    let b = B();
    a = A();
}

// The destructor in rust consists two parts to help us automatically drop all the resource owned
// by an object:
// - the programmer customized function `Drop::drop`
// - Drop glue that the compiler automatically attached for us
// Run the following example to see how it works

#[allow(unused)]
fn drop_glue1() {
    struct A(B1, B2);
    struct B1();
    struct B2();
    impl Drop for A {
        // Even if you don't have a drop implementation, drop glue still applies to release the
        // resource of its members.
        fn drop(&mut self) {
            println!("Drop for A called");
            // It is like the compiler automatically attaches sub-drop routine
            // at the end of the drop using "glue".
            println!("The following is the drop glue of A");
        }
    }
    impl Drop for B1 {
        fn drop(&mut self) {
            println!("Drop for B1 called as part of the drop glue of A");
            println!("No drop glue for B1 since it has no field");
        }
    }
    impl Drop for B2 {
        fn drop(&mut self) {
            println!("Drop for B2 called as part of the drop glue of A");
            println!("No drop glue for B2 since it has no field");
        }
    }

    A(B1(), B2());
}

// A drop glue only sticks OWNED members. If a member is a reference, the resource of it should be
// managed by its owner instead of the borrower. In simple words, who owns it drops it.
#[allow(unused)]
fn drop_glue2() {
    struct A<'a>(&'a B1, B2);
    struct B1();
    struct B2();
    impl<'a> Drop for A<'a> {
        fn drop(&mut self) {
            println!("Drop for A called");
        }
    }
    impl Drop for B1 {
        fn drop(&mut self) {
            println!("Drop for B1 called NOT as part of the drop glue of A");
            print!("Instead, this is called because its owner b1 is dropped")
        }
    }
    impl Drop for B2 {
        // I'm called as part of drop glue of A.
        fn drop(&mut self) {
            println!("Drop for B2 called as part of the drop glue of A");
        }
    }

    let b1 = B1();
    A(&b1, B2());
}

// The drop glue can process recursively if the owned member also owns a member
#[allow(unused)]
fn drop_glue3() {
    struct A(B1);
    struct B1(C1, C2);
    struct C1();
    struct C2();
    impl Drop for A {
        fn drop(&mut self) {
            println!("Drop for A called");
        }
    }
    impl Drop for B1 {
        fn drop(&mut self) {
            println!("Drop for B1 as part of the drop glue of A");
            println!("because the ownership of b1 is transferred to A")
        }
    }
    impl Drop for C1 {
        fn drop(&mut self) {
            println!("Drop for C1 as part of the drop glue of B1");
        }
    }
    impl Drop for C2 {
        fn drop(&mut self) {
            println!("Drop for C1 as part of the drop glue of B1");
        }
    }

    let b1 = B1(C1(), C2());
    A(b1);
}

// For more details about how drop glue works, check [the standard
// library](https://doc.rust-lang.org/std/ops/trait.Drop.html#drop-check)

// When drop is not explicitly implemented for a type, only drop glue would run. The compiler does
// not check the lifetime for potentially dangling reference field because we can definitely ensure that
// we would not access these reference fields during dropping. We do access owned members though.
// But that's fine because we have full privilege over it.
// We may call this kind of drop without implementation as `trivial drop`.
#[allow(unused)]
fn may_dangle1() {
    struct A<'a>(&'a B);
    struct B(i32);

    let a;
    let b = B(42);
    a = A(&b);
    drop(b); // &b dangles henceforth
             // destruct a here
             // The dangling &b doesn't matter because this is a trivial drop which means we will
             // not visit &a in destruction at all. So the compiler lets it go.
}

// When drop is explicitly implemented for a type, it requires this type outlives its reference
// even though it is not necessary to do that semantically.
#[allow(unused)]
fn may_dangle2() {
    struct A(i32);
    struct B<'a>(&'a A);
    impl<'a> Drop for B<'a> {
        fn drop(&mut self) {
            // even do nothing here
        }
    }

    // uncomment te following three lines to what will happen
    // let mut b;
    // let a = A(42);
    // b = B(&a);

    // a gets dropped here
    // b gets dropped here. a does not live long enough but we will not deref &a. So it should be
    // OK effectively.
}

// #[may_dangle] hints the compiler not to check the lifetime/borrow of 'a
#[allow(unused)]
fn may_dangle3() {
    struct A(i32);
    struct B<'a>(&'a A);
    unsafe impl<#[may_dangle] 'a> Drop for B<'a> {
        fn drop(&mut self) {
            // nothing here
        }
    }

    let mut b;
    let a = A(42);
    b = B(&a);
    // a gets dropped here
    // b gets dropped here. But #[may_dangle] makes it compiles.
}

// #[may_dangle] can be used to modify generics (after all, lifetime annotation is a variety of
// generics)
#[allow(unused)]
fn may_dangle4() {
    struct A(i32);
    struct B<T>(T);
    unsafe impl<#[may_dangle] T> Drop for B<T> {
        // impl<T> Drop for B<T> { // What happens if you uncomment this line?
        fn drop(&mut self) {
            // nothing here
        }
    }

    let mut b;
    let a = A(42);
    b = B(&a);
    // a gets dropped here
    // b gets dropped here. But #[may_dangle] makes it compiles.
}

// #[may_dangle] is unsafe which means you need to ensure that you would not use deref &A to avoid
// undefined behaviors.
#[allow(unused)]
fn may_dangle5() {
    struct B<T: Debug>(T);
    unsafe impl<#[may_dangle] T: Debug> Drop for B<T> {
        fn drop(&mut self) {
            // Warning! You told the compiler that you would not use T again but you did!
            // You'd probably get a random number other than 42.
            // It is 0 or 1 on my machine.
            println!("{:?}", self.0);
        }
    }

    let mut b;
    let a = Box::new(42);
    b = B(&a);
    // a gets dropped here
    // b gets dropped here. But #[may_dangle] makes it compiles.
}

// #[may_dangle] only applies to compiler-generated drop. If you drop variables
// explicitly, drop checks still remain.
#[allow(unused)]
fn may_dangle6() {
    struct B<T: Debug>(T);
    unsafe impl<#[may_dangle] T: Debug> Drop for B<T> {
        fn drop(&mut self) {
            // nothing here
        }
    }

    let mut b;
    let a = Box::new(42);
    b = B(&a);
    // Uncomment the following two lines to check it out.
    // drop(a);
    // drop(b);
}

// The following is another example of #[may_dangle] that shows this hint only applies in
// compileer-generated drop. Dropping the owned  or referenced field directly in your
// code makes it so dangerous that rustc decides not to indulge #[may_dangle].
#[allow(unused)]
fn may_dangle7() {
    struct A();
    struct B<T>(T); // T, which turns out to be A, is owned by B
    unsafe impl<#[may_dangle] T> Drop for B<T> {
        // impl<T> Drop for B<T> { // What happens if you uncomment this line?
        fn drop(&mut self) {
            println!("B dropped");
        }
    }
    impl Drop for A {
        fn drop(&mut self) {
            println!("A dropped as part of the drop glue of B");
        }
    }

    let mut b;
    let a = A();
    b = B(a);
    // Explicitly dropping a breaches the ownership system.
    // drop(a); // Try to uncomment this line to see the error.
}

// Everything seems perfect right now -- except only one issue left.
// To see the subtle pitfall, let us write a Box-like struct ourselves.
#[allow(unused)]
fn phantom1() {
    struct MyBox<T>(*mut T); // T is not owned by MyVec because it's a pointer.
    unsafe impl<#[may_dangle] T> Drop for MyBox<T> {
        fn drop(&mut self) {
            // If T is a reference type, simply free all the allocated space. We don't have to
            // bother with dropping them since they are borrowed instead of owned. That's why we
            // are going to add #[may_dangle] for the same reason as shown above.
            // ptr::drop_in_place would do nothing to a reference. That's fine. references does
            // nothing except for dropping the memory to store those references.
            unsafe {
                ptr::drop_in_place(self.0);
                alloc::dealloc(self.0 as *mut u8, Layout::new::<T>())
            };
        }
    }
    impl<T> MyBox<T> {
        fn new() -> MyBox<T> {
            MyBox(unsafe { alloc::alloc(Layout::new::<T>()) } as *mut T)
        }
        fn move_in(&mut self, mut t: T) {
            unsafe {
                ptr::swap(self.0, &mut t as *mut T);
            }
        }
    }

    let mut a = MyBox::new();
    let s = String::from("233");
    a.move_in(&s);
    drop(s);
    // a get dropped here where &s is dangling
}

// On the other hand, if T is actually OWNED by MyBox like MyBox<T>, we must drop all of them
// individually. So we really need to have mutable access to them and we hope our
// compiler check works in such cases. However, #[may_dangle] skims through
// the definition of struct MyVec<T> and say "T is not owned by MyVec. So I should take
// effect", which makes the compiler ignores those checks.
#[allow(unused)]
fn phantom2() {
    struct MyBox<T>(*mut T);
    struct PrintOnDrop<'s>(&'s str);
    impl<'s> Drop for PrintOnDrop<'s> {
        fn drop(&mut self) {
            println!("PrintOnDrop dropped as part of drop glue of MyBox");
            println!("visit a dangling reference: {}", self.0);
        }
    }
    unsafe impl<#[may_dangle] T> Drop for MyBox<T> {
        fn drop(&mut self) {
            println!("MyBox dropped");
            unsafe {
                ptr::drop_in_place(self.0);
                alloc::dealloc(self.0 as *mut u8, Layout::new::<T>())
            };
        }
    }
    impl<T> MyBox<T> {
        fn new() -> MyBox<T> {
            MyBox(unsafe { alloc::alloc(Layout::new::<T>()) } as *mut T)
        }
        fn move_in(&mut self, mut t: T) {
            unsafe {
                std::mem::forget(std::mem::replace(&mut *self.0, t));
            }
        }
    }

    let mut a = MyBox::new();
    let mut s = "Hello".to_owned();
    s.push_str(" world");
    a.move_in(PrintOnDrop(s.as_str()));
    drop(s);
    println!("s dropped");
    // run the code to see the output.
}

// The point to resolve this trouble is to make T owned by MyVec in some way. Something tricky like
// a zero-sized array in C language might have it settled but in rust we have a dedicated type,
// PhantomData<T>, for this.
#[allow(unused)]
fn phantom3() {
    struct MyBox<T>(*mut T, PhantomData<T>);
    struct PrintOnDrop<'s>(&'s str);
    impl<'s> Drop for PrintOnDrop<'s> {
        fn drop(&mut self) {
            println!("PrintOnDrop dropped as part of drop glue of MyBox");
            println!("visit a dangling reference: {}", self.0);
        }
    }
    unsafe impl<#[may_dangle] T> Drop for MyBox<T> {
        fn drop(&mut self) {
            println!("MyBox dropped");
            unsafe {
                ptr::drop_in_place(self.0);
                alloc::dealloc(self.0 as *mut u8, Layout::new::<T>())
            };
        }
    }
    impl<T> MyBox<T> {
        fn new() -> MyBox<T> {
            MyBox(
                unsafe { alloc::alloc(Layout::new::<T>()) } as *mut T,
                PhantomData::default(),
            )
        }
        fn move_in(&mut self, mut t: T) {
            unsafe {
                std::mem::forget(std::mem::replace(&mut *self.0, t));
            }
        }
    }

    // uncomment the following lines to see what happens;
    // let mut a = MyBox::new();
    // let mut s = "Hello".to_owned();
    // s.push_str(" world");
    // a.move_in(PrintOnDrop(s.as_str()));
    // drop(s);
    // println!("s dropped");
}

pub fn main() {
    // Uncomment them to run
    drop_order();
    // drop_glue1();
    // drop_glue2();
    // drop_glue3();
    // may_dangle1();
    // may_dangle2();
    // may_dangle3();
    // may_dangle4();
    // may_dangle5();
    // may_dangle6();
    // may_dangle7();
    // phantom1();
    // phantom2();
    // phantom3();
}
