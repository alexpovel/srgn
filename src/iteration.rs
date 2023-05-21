use itertools::Itertools;

pub fn power_set<C, T>(collection: C, include_empty_set: bool) -> Vec<Vec<T>>
where
    C: IntoIterator<Item = T>,
    T: Clone,
{
    let vec = collection.into_iter().collect_vec();

    // https://en.wikipedia.org/wiki/Power_set#Properties
    let mut result = Vec::with_capacity(2usize.checked_pow(vec.len() as u32).expect("Overflow"));

    let start = if include_empty_set { 0 } else { 1 };

    for i in start..=vec.len() {
        result.extend(vec.iter().cloned().combinations(i));
    }

    result
}

#[cfg(test)]
mod tests {
    fn sanitize_filename(filename: &str) -> String {
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

    use super::power_set;
    use rstest::rstest;
    use serde::Serialize;

    macro_rules! instrament {
        ($(#[$attr:meta])* fn $name:ident ( $( $(#[$arg_attr:meta])* $arg:ident : $type:ty),* ) $body:expr ) => {
            paste::paste! {
                #[derive(Serialize)]
                struct [<$name:camel>]<'a> {
                    $( $arg: &'a $type, )*
                }

                impl<'a> std::fmt::Display for [<$name:camel>]<'a> {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        $(
                            let mut str_val = format!("{:#?}", self.$arg);
                            str_val = sanitize_filename(&str_val);
                            println!("{}", str_val);
                            write!(f, "{} ", str_val)?;
                        )*
                        Ok(())
                    }
                }

                $(#[$attr])*
                fn $name ( $( $(#[$arg_attr])* $arg : $type),* ) {
                    let function_data = [<$name:camel>] { $($arg: &$arg),* };
                    let mut settings = insta::Settings::clone_current();
                    settings.set_info(&function_data);

                    settings.bind(|| {
                        #[allow(clippy::redundant_closure_call)]
                        $body(&function_data);
                    });
                }
            }
        };
    }

    instrament! {
        #[rstest]
        fn test_power_set(
            #[values(vec![], vec![1], vec![1, 2])]
            collection: Vec<i32>,
            #[values(true, false)]
            include_empty_set: bool
        ) (|data: &TestPowerSet| {
            let result = power_set(collection.clone(), include_empty_set);
            insta::assert_yaml_snapshot!(data.to_string(), result);
        })
    }
}
