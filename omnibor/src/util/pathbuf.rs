#[macro_export]
#[doc(hidden)]
macro_rules! pathbuf {
    ( $( $part:expr ),* ) => {{
        use std::path::PathBuf;

        let mut temp = PathBuf::new();

        $(
            temp.push($part);
        )*

        temp
    }};

    ($( $part:expr, )*) => ($crate::pathbuf![$($part),*])
}
