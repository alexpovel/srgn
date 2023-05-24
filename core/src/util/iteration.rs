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
    use super::power_set;
    use instrament::instrament;
    use rstest::rstest;

    instrament! {
        #[rstest]
        fn test_power_set(
            #[values(vec![], vec![1], vec![1, 2], vec![1, 2, 3])]
            collection: Vec<i32>,
            #[values(true, false)]
            include_empty_set: bool
        ) (|data: &TestPowerSet| {
            let result = power_set(collection.clone(), include_empty_set);
            insta::assert_yaml_snapshot!(data.to_string(), result);
        })
    }
}
