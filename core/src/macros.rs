/// Sanitize a string for use as a filename.
///
/// A 'best effort' in replacing characters illegal in filenames with a stand-in
/// [`char`]. For Unix, it's easy, for Windows, it's a bit of guessing. Use with
/// caution.
#[must_use]
pub fn sanitize_for_filename_use(filename: &str) -> String {
    const REPLACEMENT: char = '_';
    filename
        .replace(
            [
                ' ', ':', '<', '>', '\"', '/', '\\', '|', '?', '*', '\n', '\r',
            ],
            &REPLACEMENT.to_string(),
        )
        // Collapse consecutive underscores into one
        .split(REPLACEMENT)
        .filter(|&s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(&REPLACEMENT.to_string())
}

/// This macro allows `rstest` (test case parametrization) and `insta` (snapshot
/// testing) to coexist.
///
/// The gist is that it generates a whole new `struct` from a function signature.
/// Implementing [`std::fmt::Display`] for it, the struct can be used to represent a
/// test case automatically: whatever was passed into the test function by `rstest` and
/// its `case`s will form the `struct` fields, which form the name. As a result, all
/// tests get unique, easily identified names.
///
/// One caveat is that a closure is now required to run a test case. Quite ugly. In
/// general, **do not use this macro** if you can avoid it. Consider this macro as
/// potentially breaking at any point. When using *either* `rstest` *or* `insta`, this
/// macro is not needed.
///
/// For context, see [this
/// issue](https://github.com/la10736/rstest/issues/183#issuecomment-1564021215).
#[macro_export]
macro_rules! instrament {
    ($(#[$attr:meta])* fn $name:ident ( $( $(#[$arg_attr:meta])* $arg:ident : $type:ty),* ) $body:expr ) => {
        ::paste::paste! {
            #[derive(::serde::Serialize)]
            struct [<$name:camel>]<'a> {
                $( $arg: &'a $type, )*
            }

            impl<'a> ::std::fmt::Display for [<$name:camel>]<'a> {
                fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                    let items: Vec<String> = vec![
                        stringify!($name).to_owned() ,
                        $(
                            format!("{:#?}", self.$arg).to_owned() ,
                        )*
                    ];

                    let name = $crate::macros::sanitize_for_filename_use(&items.join("-"));
                    write!(f, "{}", name)
                }
            }

            $(#[$attr])*
            fn $name ( $( $(#[$arg_attr])* $arg : $type),* ) {
                let function_data = [<$name:camel>] { $($arg: &$arg),* };
                let mut settings = ::insta::Settings::clone_current();
                settings.set_info(&function_data);

                settings.bind(|| {
                    #[allow(clippy::redundant_closure_call)]
                    $body(&function_data);
                });
            }
        }
    };
}
