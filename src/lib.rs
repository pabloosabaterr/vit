pub mod commit;
pub mod config;
pub mod error;
pub mod git;
pub(crate) mod lin_alg;
pub mod lsa;
pub(crate) mod read;
pub(crate) mod sparse_matrix;
pub(crate) mod stemmer;
pub(crate) mod term;
pub mod text;
pub mod vector;
pub mod word_map;

#[macro_export]
macro_rules! verbose {
    ($verbose: expr, $($arg:tt)*) => {
        if $verbose {
            eprintln!($($arg)*)
        }
    };
}

#[macro_export]
macro_rules! die {
    ($($arg:tt)*) => {{
        eprintln!("fatal: {}", format!($($arg)*));
        std::process::exit(1);
    }};
}

/*
 * Version format: a.b.c.d
 *
 * a: major, rarely changes
 * b: breaking change in .vit/ format
 * c: features and relevant changes
 * d: weekly release (week number of the year)
 *
 * Only a and b are checked when reading .vit/ files.
 */
pub const VERSION: [u8; 4] = [
    env!("CARGO_PKG_VERSION_MAJOR").as_bytes()[0] - b'0',
    env!("CARGO_PKG_VERSION_MINOR").as_bytes()[0] - b'0',
    env!("CARGO_PKG_VERSION_PATCH").as_bytes()[0] - b'0',
    26,
];
