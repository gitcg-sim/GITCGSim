/// Property tests of paying Elemental Dice costs.
///
/// Loosening transformations: Transformations on the dice or cost to make the cost easier to pay (outcome = pass/fail for paying the cost).
///  - Adding more dice preserves the outcome
///  - Converting dice into Omni preserves the outcome
///  - Updating the ElementPriority preserves the outcome
///
/// TODO determine if Omni dice should be minimized at all costs (paying preferred only vs. Omni + non-referred)
/// Important example: Dice = Omni 1, Pyro 1, Dendro 2, cost = Aligned 2
/// If Dendro is preferred over Pyro, then Pyro 1, Omni 1 will be spent
/// Otherwise, Pyro 2 will be spent
use crate::elem_set;

use super::*;
use proptest::prelude::*;

prop_compose! {
    pub fn arb_elem()(v in any::<u8>()) -> Element {
        Element::VALUES[(v as usize) % Element::VALUES.len()]
    }
}

prop_compose! {
    pub fn arb_dice()(v in any::<u8>()) -> Dice {
        Dice::VALUES[(v as usize) % Dice::VALUES.len()]
    }
}

prop_compose! {
    pub fn arb_dice_counter_free()(vs in any::<[u8; 8]>()) -> DiceCounter {
        let mut dice_counter = DiceCounter::default();
        for (d, v) in Dice::VALUES.iter().copied().zip(vs) {
            dice_counter.set_single(d, v & DiceCounter::MASK);
        }
        dice_counter
    }
}

prop_compose! {
    pub fn arb_dice_counter_single()(d in arb_dice(), v in any::<u8>()) -> DiceCounter {
        let mut dice_counter = DiceCounter::default();
        dice_counter.set_single(d, v & DiceCounter::MASK);
        dice_counter
    }
}

prop_compose! {
    pub fn arb_dice_counter_single_omni()(d in arb_dice(), (o, v) in any::<(u8, u8)>()) -> DiceCounter {
        let mut dice_counter = DiceCounter::default();
        dice_counter.set_single(Dice::Omni, o & DiceCounter::MASK);
        dice_counter.set_single(d, v & DiceCounter::MASK);
        dice_counter
    }
}

prop_compose! {
    pub fn arb_dice_counter_double_omni()(
        d1 in arb_dice(), d2 in arb_dice(), (o, v1, v2) in any::<(u8, u8, u8)>()
    ) -> DiceCounter {
        let mut dice_counter = DiceCounter::default();
        dice_counter.set_single(Dice::Omni, o & DiceCounter::MASK);
        dice_counter.set_single(d1, v1 & DiceCounter::MASK);
        dice_counter.set_single(d2, v2 & DiceCounter::MASK);
        dice_counter
    }
}

pub fn arb_dice_counter() -> impl Strategy<Value = DiceCounter> {
    prop_oneof![
        Just(Default::default()),
        arb_dice_counter_free(),
        arb_dice_counter_single(),
        arb_dice_counter_single_omni(),
        arb_dice_counter_double_omni(),
    ]
}

pub fn arb_tcg_cost_no_energy() -> impl Strategy<Value = Cost> {
    let range = || 0u8..5u8;
    prop_oneof![
        range().prop_map(Cost::aligned),
        range().prop_map(Cost::unaligned),
        (arb_elem(), range()).prop_map(|(e, k)| Cost::elem(e, k)),
        (arb_elem(), range(), range()).prop_map(|(e, k, a)| Cost::elem(e, k).with_unaligned(a)),
        (arb_elem(), range(), range()).prop_map(|(e, k, a)| Cost::elem(e, k).with_aligned(a)),
    ]
}

impl Cost {
    pub fn loosen_to_unaligned(&self) -> Self {
        Self::unaligned(self.elem_cost.map(|(_, v)| v).unwrap_or_default() + self.unaligned_cost + self.aligned_cost)
    }
}

prop_compose! {
    /// Returns `(d1, d2)` where d2 is a superset of d1 (same or more multiplicity for each dice).
    pub fn arb_dice_counter_superset()(
        d1 in arb_dice_counter(),
        d2 in arb_dice_counter(),
    ) -> (DiceCounter, DiceCounter) {
        let mut d = d1;
        d.add_tally(d2.tally());
        (d1, d)
    }
}

prop_compose! {
    pub fn arb_element_priority()(
        (e1, e2, e3, e4, e5) in (arb_elem(), arb_elem(), arb_elem(), arb_elem(), arb_elem()),
        has_active in any::<u8>(),
    ) -> ElementPriority {
        ElementPriority {
            important_elems: elem_set![e1, e2, e3, e4],
            active_elem: if has_active % 3 == 0 { None } else { Some(e5) },
        }
    }
}

impl ElementPriority {
    pub fn downgrade_elem(&mut self, elem: Element) -> bool {
        let mut changed = false;
        if self.active_elem == Some(elem) {
            self.active_elem = None;
            self.important_elems.insert(elem);
            changed = true;
        }
        if self.important_elems.contains(elem) {
            self.important_elems.remove(elem);
            changed = true;
        }
        changed
    }
}

pub fn arb_element_priority_with_downgrade() -> impl Strategy<Value = (ElementPriority, ElementPriority, Element)> {
    (arb_element_priority(), any::<usize>()).prop_map(|(ep, i)| {
        let elems: smallvec::SmallVec<[Element; 8]> = ep.elems().iter().collect();
        let elem = elems[i % elems.len()];
        let mut ep1 = ep;
        ep1.downgrade_elem(elem);
        (ep, ep1, elem)
    })
}

pub fn cost_props<F: Fn(DiceCounter, DiceCounter) -> R, R>(
    (d1, cost1): (DiceCounter, Cost),
    (d2, cost2): (DiceCounter, Cost),
    (ep1, ep2): (ElementPriority, ElementPriority),
    f: F,
) -> Result<Option<(R, R)>, std::convert::Infallible> {
    let Some(updated) = d1.try_pay_cost(&cost1, &ep1) else {
        return Ok(None);
    };
    let a = f(d1, updated);
    let updated1 = d2.try_pay_cost(&cost2, &ep2).expect("property failed");
    let b = f(d2, updated1);
    Ok(Some((a, b)))
}

pub fn ensure_does_not_increase_omni_paid(
    t1: (DiceCounter, Cost),
    t2: (DiceCounter, Cost),
    ep: (ElementPriority, ElementPriority),
) -> Result<(), std::convert::Infallible> {
    let Some((omni_paid, omni_paid_superset)) =
        cost_props(t1, t2, ep, |d, updated| d[Dice::Omni] - updated[Dice::Omni])?
    else {
        return Ok(());
    };
    assert!(omni_paid_superset <= omni_paid);
    Ok(())
}

pub fn ensure_same_number_of_dice_spent(
    t1: (DiceCounter, Cost),
    t2: (DiceCounter, Cost),
    ep: (ElementPriority, ElementPriority),
) -> Result<(), std::convert::Infallible> {
    let Some((paid, paid_superset)) = cost_props(t1, t2, ep, |d, updated| d.total() - updated.total())? else {
        return Ok(());
    };
    assert!(paid_superset == paid);
    Ok(())
}

mod different_element_priority {
    use super::*;

    proptest! {
        #[test]
        fn preserves_outcome(d in arb_dice_counter(), cost in arb_tcg_cost_no_energy(), ep1 in arb_element_priority(), ep2 in arb_element_priority()) {
            assert_eq!(
                d.try_pay_cost(&cost, &ep1).is_some(),
                d.try_pay_cost(&cost, &ep2).is_some()
            );
        }

        #[test]
        fn same_number_of_dice_spent(d in arb_dice_counter(), cost in arb_tcg_cost_no_energy(), ep1 in arb_element_priority(), ep2 in arb_element_priority()) {
            ensure_same_number_of_dice_spent((d, cost), (d, cost), (ep1, ep2))?;
        }

        // TODO doesn't work
        #[cfg(any())]
        #[test]
        fn same_number_of_omni_paid(d in arb_dice_counter(), cost in arb_tcg_cost_no_energy(), ep1 in arb_element_priority(), ep2 in arb_element_priority()) {
            let Some((omni_paid, omni_paid1)) =
                cost_props((d, cost), (d, cost), (ep1, ep2), |d, updated| d[Dice::Omni] - updated[Dice::Omni])?
            else {
                return Ok(());
            };
            assert_eq!(omni_paid1, omni_paid);
        }
    }
}

mod superset {
    use super::*;

    proptest! {
        #[test]
        fn superset_total((d1, d2) in arb_dice_counter_superset()) {
            assert!(d1.total() <= d2.total())
        }

        #[test]
        fn individual_dice_ordering((d1, d2) in arb_dice_counter_superset()) {
            for d in Dice::VALUES {
                assert!(d1[d] <= d2[d])
            }
        }

        #[test]
        fn can_pay_cost((d1, d2) in arb_dice_counter_superset(), cost in arb_tcg_cost_no_energy(), ep in arb_element_priority()) {
            if d1.try_pay_cost(&cost, &ep).is_some() {
                assert!(d2.try_pay_cost(&cost, &ep).is_some());
            }
        }

        #[test]
        fn same_number_of_dice_spent((d1, d2) in arb_dice_counter_superset(), cost in arb_tcg_cost_no_energy(), ep in arb_element_priority()) {
            ensure_same_number_of_dice_spent((d1, cost), (d2, cost), (ep, ep))?;
        }

        #[cfg(any())]
        #[test]
        fn does_not_increase_omni_paid((d1, d2) in arb_dice_counter_superset(), cost in arb_tcg_cost_no_energy(), ep in arb_element_priority()) {
            ensure_does_not_increase_omni_paid((d1, cost), (d2, cost), (ep, ep))?;
        }
    }
}

mod loosen_cost {
    use super::*;

    proptest! {
        #[test]
        fn can_pay_cost(d in arb_dice_counter(), cost in arb_tcg_cost_no_energy(), ep in arb_element_priority()) {
            if d.try_pay_cost(&cost, &ep).is_some() {
                assert!(d.try_pay_cost(&cost.loosen_to_unaligned(), &ep).is_some());
            };
        }

        #[test]
        fn same_number_of_dice_spent(d in arb_dice_counter(), cost in arb_tcg_cost_no_energy(), ep in arb_element_priority()) {
            ensure_same_number_of_dice_spent((d, cost), (d, cost.loosen_to_unaligned()), (ep, ep))?;
        }

        #[test]
        fn does_not_increase_omni_paid(d in arb_dice_counter(), cost in arb_tcg_cost_no_energy(), ep in arb_element_priority()) {
            ensure_does_not_increase_omni_paid((d, cost), (d, cost.loosen_to_unaligned()), (ep, ep))?;
        }
    }
}

mod downgrade_elem_priority {
    use super::*;

    proptest! {
        #[test]
        fn does_not_increase_omni_paid(
            d in arb_dice_counter(),
            cost in arb_tcg_cost_no_energy(),
            (ep1, ep2, _) in arb_element_priority_with_downgrade()
        ) {
            let Some((omni_paid, omni_paid_downgrade)) =
                cost_props((d, cost), (d, cost), (ep1, ep2), |d, updated| d[Dice::Omni] - updated[Dice::Omni])?
            else {
                return Ok(());
            };
            assert!(omni_paid >= omni_paid_downgrade);
        }

        #[test]
        fn does_not_decrease_downgraded_elem_paid(
            d in arb_dice_counter(),
            cost in arb_tcg_cost_no_energy(),
            (ep1, ep2, elem) in arb_element_priority_with_downgrade()
        ) {
            let Some((elem_paid, elem_paid_downgrade)) =
                cost_props((d, cost), (d, cost), (ep1, ep2), |d, updated| d[Dice::Elem(elem)] - updated[Dice::Elem(elem)])?
            else {
                return Ok(());
            };
            assert!(elem_paid <= elem_paid_downgrade);
        }

        #[test]
        fn unaligned_or_aligned_affected_by_element_priority_single_elem(n in 0..10u8, e1 in arb_elem(), e2 in arb_elem(), ep in arb_element_priority(), is_aligned in any::<bool>()) {
            prop_assume!(e1 != e2);
            let mut ep = ep;
            ep.downgrade_elem(e2);
            ep.downgrade_elem(e2);
            prop_assume!(ep.elems().contains(e1));
            let cost = if is_aligned { Cost::aligned(n) } else { Cost::unaligned(n) };
            let mut d = DiceCounter::default();
            d.add_single(Dice::Elem(e1), n);
            d.add_single(Dice::Elem(e2), n);
            let d1 = d.try_pay_cost(&cost, &ep).expect("fail");
            assert_eq!(d1[Dice::Elem(e1)], n);
            assert_eq!(d1[Dice::Elem(e2)], 0);
        }
    }
}

mod aligned_cost {
    use super::*;

    proptest! {
        #[test]
        fn zero_or_one_distinct_element_paid(d in arb_dice_counter(), n in 0..10u8, ep in arb_element_priority()) {
            let cost = Cost::aligned(n);
            let Some(paid) = d.try_pay_cost(&cost, &ep) else { prop_assume!(false); unreachable!(); };
            let elems_paid: ElementSet = Element::VALUES.iter().copied().filter(|&e| d[Dice::Elem(e)] - paid[Dice::Elem(e)] > 0).collect();
            assert!(elems_paid.len() <= 1);
        }

        #[test]
        fn total_dice_paid(d in arb_dice_counter(), n in 0..10u8, ep in arb_element_priority()) {
            let cost = Cost::aligned(n);
            let Some(paid) = d.try_pay_cost(&cost, &ep) else { prop_assume!(false); unreachable!(); };
            let diff = d.total() - paid.total();
            assert_eq!(diff, n);
        }
    }
}

mod unaligned_cost {
    use super::*;

    proptest! {
        #[test]
        fn total_dice_paid(d in arb_dice_counter(), n in 0..10u8, ep in arb_element_priority()) {
            let cost = Cost::unaligned(n);
            let Some(paid) = d.try_pay_cost(&cost, &ep) else { prop_assume!(false); unreachable!(); };
            let diff = d.total() - paid.total();
            assert_eq!(diff, n);
        }

        #[test]
        fn preferred_elem_not_paid_if_possible(d in arb_dice_counter(), n in 0..10u8, ep in arb_element_priority(), e in arb_elem()) {
            let mut d = d;
            d.set_single(Dice::Elem(e), 0);
            prop_assume!(ep.elems().contains(e));
            prop_assume!(d.total() - d[Dice::Elem(e)] >= n);
            let cost = Cost::unaligned(n);
            let Some(paid) = d.try_pay_cost(&cost, &ep) else { prop_assume!(false); unreachable!(); };
            assert_eq!(d[Dice::Elem(e)], paid[Dice::Elem(e)]);
        }
    }
}

mod elem_cost {
    use super::*;

    proptest! {
        #[test]
        fn total_dice_paid(d in arb_dice_counter(), e in arb_elem(), n in 0..10u8, ep in arb_element_priority()) {
            let cost = Cost::elem(e, n);
            let Some(paid) = d.try_pay_cost(&cost, &ep) else { prop_assume!(false); unreachable!(); };
            let diff = d.total() - paid.total();
            assert_eq!(diff, n);
        }

        #[test]
        fn only_element_paid(d in arb_dice_counter(), e in arb_elem(), n in 0..10u8, ep in arb_element_priority()) {
            let cost = Cost::elem(e, n);
            let Some(paid) = d.try_pay_cost(&cost, &ep) else { prop_assume!(false); unreachable!(); };
            let elems_paid: ElementSet = Element::VALUES.iter().copied().filter(|&e| d[Dice::Elem(e)] - paid[Dice::Elem(e)] > 0).collect();
            if !elems_paid.is_empty() {
                assert_eq!(elems_paid, elem_set![e]);
            }
        }

        #[test]
        fn not_affected_by_element_priority(d in arb_dice_counter(), n in 0..10u8, (ep1, ep2, e) in arb_element_priority_with_downgrade()) {
            let cost = Cost::elem(e, n);
            assert_eq!(d.try_pay_cost(&cost, &ep1), d.try_pay_cost(&cost, &ep2));
        }
    }
}

// TODO ElementPriority tests: A + B where A contains EP elements and B does not at all
// TODO ELementPriority: more-preferred properties
