use itertools::Itertools;

pub fn power_set<C, T>(collection: C) -> Vec<Vec<T>>
where
    C: IntoIterator<Item = T>,
    T: Clone,
{
    let vec = collection.into_iter().collect_vec();

    // https://en.wikipedia.org/wiki/Power_set#Properties
    let mut result = Vec::with_capacity(2usize.checked_pow(vec.len() as u32).expect("Overflow"));

    for i in 0..=vec.len() {
        result.extend(vec.iter().cloned().combinations(i));
    }

    result
}

#[cfg(test)]
mod tests {
    use super::power_set;
    use rstest::rstest;

    type TestVec = Vec<i32>;

    #[rstest]
    #[case(Vec::new(), vec![vec![]])]
    #[case(vec![1], vec![vec![], vec![1]])]
    #[case(vec![1, 2], vec![vec![], vec![1], vec![2], vec![1, 2]])]
    #[case(
        vec![1, 2, 3],
        vec![
            vec![],
            vec![1],
            vec![2],
            vec![3],
            vec![1, 2],
            vec![1, 3],
            vec![2, 3],
            vec![1, 2, 3]
        ]
    )]
    fn test_powerset_of_integers(#[case] input: TestVec, #[case] expected: Vec<TestVec>) {
        let result: Vec<Vec<i32>> = power_set(input);
        assert_eq!(result, expected);
    }

    #[rstest]
    fn test_powerset_of_tuples() {
        let input = vec![(1, 2), (2, 4), (3, 9)];
        let expected = vec![
            vec![],
            vec![(1, 2)],
            vec![(2, 4)],
            vec![(3, 9)],
            vec![(1, 2), (2, 4)],
            vec![(1, 2), (3, 9)],
            vec![(2, 4), (3, 9)],
            vec![(1, 2), (2, 4), (3, 9)],
        ];

        let result = power_set(input);
        assert_eq!(result, expected);
    }
}
