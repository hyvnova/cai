  // for write!()


/// Concatenate any number of `Display` items into a single `String`.
/// Usage:
/// ```rust
/// let s = text!(
///     "[ERROR]".bold().red(),
///     "\n\t",
///     err.to_string().red()
/// );
/// println!("{s}");
/// ```
#[macro_export]
macro_rules! text {
    ( $first:expr $(, $rest:expr )* $(,)? ) => {{
        use std::fmt::Write; 
        let mut s = String::new();
        write!(&mut s, "{}", $first).unwrap();
        $(
            s.push('\n');
            write!(&mut s, "{}", $rest).unwrap();
        )*
        s
    }};
}