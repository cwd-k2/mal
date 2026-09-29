use super::*;

#[test]
fn canonicalizes_long_alias_chains_without_host_recursion() {
    const ALIAS_COUNT: u32 = 50_000;
    let leaf = ValueId::Temporary(ALIAS_COUNT);
    let mut authorities = (0..ALIAS_COUNT)
        .map(|index| {
            (
                ValueId::Temporary(index),
                HashSet::from([ValueId::Temporary(index + 1)]),
            )
        })
        .collect::<HashMap<_, _>>();

    canonicalize(&mut authorities);

    assert_eq!(authorities.len(), ALIAS_COUNT as usize);
    assert!(
        authorities
            .values()
            .all(|lenders| lenders == &HashSet::from([leaf]))
    );
}

#[test]
fn discards_a_cycle_without_recursion() {
    let left = ValueId::Temporary(0);
    let right = ValueId::Temporary(1);
    let mut authorities = HashMap::from([
        (left, HashSet::from([right])),
        (right, HashSet::from([left])),
    ]);

    canonicalize(&mut authorities);

    assert!(authorities.is_empty());
}
