use itertools::Itertools;

pub fn _power_set<C, T>(collection: C) -> Vec<Vec<T>>
where
    C: IntoIterator<Item = T>,
    T: Clone,
{
    power_set_impl(collection, true)
}

pub fn power_set_without_empty<C, T>(collection: C) -> Vec<Vec<T>>
where
    C: IntoIterator<Item = T>,
    T: Clone,
{
    power_set_impl(collection, false)
}

fn power_set_impl<C, T>(collection: C, include_empty_set: bool) -> Vec<Vec<T>>
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
    use super::{_power_set, power_set_without_empty};
    use crate::instrament;
    use rstest::rstest;

    instrament! {
        #[rstest]
        fn test_power_set(
            #[values(vec![], vec![1], vec![1, 2], vec![1, 2, 3])]
            collection: Vec<i32>
        ) (|data: &TestPowerSet| {
            let result = _power_set(collection.clone());
            insta::assert_yaml_snapshot!(data.to_string(), result);
        })
    }

    instrament! {
        #[rstest]
        fn test_power_set_without_empty(
            #[values(vec![], vec![1], vec![1, 2], vec![1, 2, 3])]
            collection: Vec<i32>
        ) (|data: &TestPowerSetWithoutEmpty| {
            let result = power_set_without_empty(collection.clone());
            insta::assert_yaml_snapshot!(data.to_string(), result);
        })
    }
}
