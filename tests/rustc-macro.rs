extern crate cargotest;
extern crate hamcrest;

use cargotest::is_nightly;
use cargotest::support::{project, execs};
use hamcrest::assert_that;

#[test]
fn noop() {
    if !is_nightly() {
        return;
    }

    let client = project("client")
        .file("Cargo.toml", r#"
            [package]
            name = "client"
            version = "0.0.1"
            authors = []

            [dependencies.noop]
            path = "../noop"
        "#)
        .file("src/main.rs", r#"
            #![feature(rustc_macro)]

            #[macro_use]
            extern crate noop;

            #[derive(Noop)]
            struct X;

            fn main() {}
        "#);
    let noop = project("noop")
        .file("Cargo.toml", r#"
            [package]
            name = "noop"
            version = "0.0.1"
            authors = []

            [lib]
            rustc-macro = true
        "#)
        .file("src/lib.rs", r#"
            #![feature(rustc_macro, rustc_macro_lib)]

            extern crate rustc_macro;
            use rustc_macro::TokenStream;

            #[rustc_macro_derive(Noop)]
            pub fn noop(input: TokenStream) -> TokenStream {
                input
            }
        "#);
    noop.build();

    assert_that(client.cargo_process("build"),
                execs().with_status(0));
    assert_that(client.cargo("build"),
                execs().with_status(0));
}

#[test]
fn impl_and_derive() {
    if !is_nightly() {
        return;
    }

    let client = project("client")
        .file("Cargo.toml", r#"
            [package]
            name = "client"
            version = "0.0.1"
            authors = []

            [dependencies.transmogrify]
            path = "../transmogrify"
        "#)
        .file("src/main.rs", r#"
            #![feature(rustc_macro)]

            #[macro_use]
            extern crate transmogrify;

            trait ImplByTransmogrify {
                fn impl_by_transmogrify(&self) -> bool;
            }

            #[derive(Transmogrify)]
            struct X;

            fn main() {
                let x = X::new();
                assert!(x.impl_by_transmogrify());
                println!("{:?}", x);
            }
        "#);
    let transmogrify = project("transmogrify")
        .file("Cargo.toml", r#"
            [package]
            name = "transmogrify"
            version = "0.0.1"
            authors = []

            [lib]
            rustc-macro = true
        "#)
        .file("src/lib.rs", r#"
            #![feature(rustc_macro, rustc_macro_lib)]

            extern crate rustc_macro;
            use rustc_macro::TokenStream;

            #[rustc_macro_derive(Transmogrify)]
            #[doc(hidden)]
            pub fn transmogrify(input: TokenStream) -> TokenStream {
                assert_eq!(input.to_string(), "struct X;\n");

                "
                    impl X {
                        fn new() -> Self {
                            X { success: true }
                        }
                    }

                    impl ImplByTransmogrify for X {
                        fn impl_by_transmogrify(&self) -> bool {
                            true
                        }
                    }

                    #[derive(Debug)]
                    struct X {
                        success: bool,
                    }
                ".parse().unwrap()
            }
        "#);
    transmogrify.build();

    assert_that(client.cargo_process("build"),
                execs().with_status(0));
    assert_that(client.cargo("run"),
                execs().with_status(0).with_stdout("X { success: true }"));
}
