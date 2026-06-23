pub mod commit;
pub mod config;
pub mod git;
pub mod lin_alg;
pub mod lsa;
pub mod sparse_matrix;
pub mod stemmer;
pub mod term;
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
