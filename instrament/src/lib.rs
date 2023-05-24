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

                    let name = $crate::sanitize_for_filename_use(&items.join("-"));
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
