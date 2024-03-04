use crate::std_subset::{ops::Index, vec, Vec};

use constdefault::ConstDefault;
use enumset::enum_set;
use smallvec::SmallVec;

use crate::types::ElementSet;
use crate::types::{
    card_defs::Cost,
    tcg_model::{Dice, Element},
};

mod distribution;
pub use distribution::{DiceDeterminization, DiceDistribution};

pub(crate) mod builder;

/// Represents the collection of Elemental Dice (Omni and the 7 elements).
/// The maximum number of dice for a particular element (or Omni) is 31.
#[derive(Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(from = "builder::DiceCounterBuilder"),
    serde(into = "builder::DiceCounterBuilder")
)]
pub struct DiceCounter {
    omni: u8,
    elem: [u8; 7],
}

impl crate::std_subset::fmt::Debug for DiceCounter {
    fn fmt(&self, f: &mut crate::std_subset::fmt::Formatter<'_>) -> crate::std_subset::fmt::Result {
        f.debug_tuple("DiceCounter").field(&self.tally()).finish()
    }
}

/// Describes the preferred elements to keep for selecting Elemental Dice for
/// (1) cost payments and (2) automatic dice rerolls
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ElementPriority {
    /// Preferred elements (lower priority)
    pub important_elems: ElementSet,
    /// Element of the active character, if applicable (higher priority)
    pub active_elem: Option<Element>,
}

impl ConstDefault for ElementPriority {
    const DEFAULT: Self = Self {
        important_elems: enum_set![],
        active_elem: None,
    };
}

impl Default for ElementPriority {
    fn default() -> Self {
        ConstDefault::DEFAULT
    }
}

impl ElementPriority {
    #[inline]
    pub fn new(important_elems: ElementSet, active_elem: Option<Element>) -> ElementPriority {
        ElementPriority {
            important_elems,
            active_elem,
        }
    }

    #[inline]
    pub fn elems(&self) -> ElementSet {
        if let Some(e) = self.active_elem {
            let mut es = self.important_elems;
            es.insert(e);
            es
        } else {
            self.important_elems
        }
    }

    /// Get the elements outside the priority
    #[inline]
    pub fn off_elems(&self) -> ElementSet {
        let mut es = self.important_elems;
        if let Some(e) = self.active_elem {
            es |= e;
        }
        !es
    }

    /// Get the first element (lowest position in `Element::ALL`) that is not part of this priority.
    #[inline]
    pub fn get_off_element(&self) -> Option<Element> {
        let mut elems = self.important_elems;
        if let Some(e) = self.active_elem {
            elems.insert(e);
        }
        Element::VALUES.iter().copied().find(|&e| elems.contains(e))
    }

    #[inline]
    pub fn sort_key(&self, dice: Dice) -> u8 {
        match dice {
            Dice::Omni => 0,
            Dice::Elem(e) => {
                if self.active_elem == Some(e) {
                    1
                } else if self.important_elems.contains(e) {
                    2
                } else {
                    3
                }
            }
        }
    }
}

macro_rules! consume {
    ($remain: ident, $field: expr) => {
        if $remain > 0 && $field > 0 {
            let m = if $field < $remain { $field } else { $remain };
            $field -= m;
            $remain -= m;
        }
    };
}

macro_rules! consume_matching {
    ($remain: ident, $field: expr, $omni: expr) => {
        if $remain > 0 {
            if $field >= $remain {
                $field -= $remain;
                $remain = 0;
            } else if $field + $omni >= $remain {
                $omni -= $remain - $field;
                $field = 0;
                $remain = 0;
            }
        }
    };
}

impl DiceCounter {
    pub const MASK: u8 = 31;
    pub const EMPTY: DiceCounter = DiceCounter {
        omni: 0,
        elem: [0, 0, 0, 0, 0, 0, 0],
    };

    pub const fn omni(omni: u8) -> DiceCounter {
        DiceCounter {
            omni,
            elem: [0, 0, 0, 0, 0, 0, 0],
        }
    }

    #[inline]
    pub const fn elem(elem: Element, count: u8) -> DiceCounter {
        let mut c = DiceCounter::EMPTY;
        c.elem[elem.to_index()] += count;
        c
    }

    pub fn new(tally: &Vec<(Dice, u8)>) -> DiceCounter {
        let mut value = DiceCounter::default();
        for (dice, c) in tally {
            match dice {
                Dice::Omni => {
                    value.omni += c;
                }
                Dice::Elem(e) => {
                    value.elem[e.to_index()] += c;
                }
            }
        }
        value
    }

    pub fn tally(&self) -> Vec<(Dice, u8)> {
        let mut t = vec![];
        if self.omni > 0 {
            t.push((Dice::Omni, self.omni));
        }
        for e in Element::VALUES {
            let v = self.elem[e.to_index()];
            if v > 0 {
                t.push((Dice::Elem(e), v));
            }
        }
        t
    }

    #[inline]
    pub fn total(&self) -> u8 {
        let mut es: u8 = self.omni;
        for i in 0..7 {
            es += self.elem[i];
        }
        es
    }

    #[inline]
    pub fn parity(&self) -> u8 {
        let mut p: u8 = self.omni & 1;
        for i in 0..7 {
            p ^= self.elem[i] & 1;
        }
        p
    }

    #[inline]
    pub fn distinct_count(&self) -> u8 {
        let c0 = if self.omni > 0 { 1 } else { 0 };
        self.elem.iter().fold(c0, |c, &e| c + if e > 0 { 1 } else { 0 })
    }

    #[inline]
    pub fn set_single(&mut self, dice: Dice, value: u8) {
        match dice {
            Dice::Omni => self.omni = value,
            Dice::Elem(e) => self.elem[e.to_index()] = value & Self::MASK,
        }
    }

    #[inline]
    pub fn add_single(&mut self, dice: Dice, increase: u8) {
        match dice {
            Dice::Omni => Self::add(&mut self.omni, increase),
            Dice::Elem(e) => Self::add(&mut self.elem[e.to_index()], increase),
        }
    }

    #[inline]
    pub fn sub_single(&mut self, dice: Dice, decrease: u8) {
        match dice {
            Dice::Omni => Self::sub(&mut self.omni, decrease),
            Dice::Elem(e) => Self::sub(&mut self.elem[e.to_index()], decrease),
        }
    }

    #[inline]
    pub fn add_tally<T: IntoIterator<Item = (Dice, u8)>>(&mut self, it: T) {
        for (dice, increase) in it.into_iter() {
            self.add_single(dice, increase);
        }
    }

    #[inline]
    pub fn add_dice(&mut self, other: &DiceCounter) {
        Self::add(&mut self.omni, other.omni);
        for i in 0..7 {
            Self::add(&mut self.elem[i], other.elem[i]);
        }
    }

    #[inline]
    pub fn subtract_dice(&mut self, other: &DiceCounter) {
        Self::sub(&mut self.omni, other.omni);
        for i in 0..7 {
            Self::sub(&mut self.elem[i], other.elem[i]);
        }
    }

    #[inline(always)]
    fn add(a: &mut u8, b: u8) {
        *a = (*a + b).min(Self::MASK);
    }

    #[inline(always)]
    fn sub(a: &mut u8, b: u8) {
        *a = a.saturating_sub(b);
    }

    #[inline]
    pub fn try_pay_cost_short(&self, cost: &Cost) -> Option<DiceCounter> {
        self.try_pay_cost(cost, &ElementPriority::default())
    }

    pub fn elem_order(&self, ep: &ElementPriority) -> SmallVec<[Element; 7]> {
        let mut elems: SmallVec<[Element; 7]> = Element::VALUES
            .iter()
            .copied()
            .filter(|&e| self.elem[e.to_index()] > 0)
            .collect();

        let ElementPriority {
            important_elems,
            active_elem,
        } = *ep;
        elems.sort_unstable_by_key(|&e| {
            (0x200_u16 - (self.elem[e.to_index()] as u16))
                + (if important_elems.contains(e) { 0x400 } else { 0 })
                + (if active_elem == Some(e) { 0x800 } else { 0 })
        });

        elems
    }

    /// Try to pay elemental dice cost using automatic dice selection, given element priority
    pub fn try_pay_cost(&self, cost: &Cost, ep: &ElementPriority) -> Option<DiceCounter> {
        let Cost {
            elem_cost: elem,
            unaligned_cost: unaligned,
            aligned_cost: matching,
            ..
        } = *cost;

        let ec = if let Some((e, v)) = elem {
            if v > self.omni + self.elem[e.to_index()] {
                return None;
            }
            v
        } else {
            0
        };

        let mut tot = self.total();
        if ec + unaligned + matching > tot {
            return None;
        }

        let mut updated = *self;

        // Pay elemental costs
        if let Some((e, v)) = elem {
            let mut remain = v;
            {
                let i = e.to_index();
                consume_matching!(remain, updated.elem[i], updated.omni);
            }

            if remain > 0 {
                return None;
            }

            tot -= v;
        }

        if unaligned + matching > tot {
            return None;
        }

        // Pay matching costs
        if matching > 0 {
            let mut remain = matching;

            // Pay matching costs with Omni
            if remain > 0 {
                for e in updated.elem_order(ep) {
                    let i = e.to_index();
                    consume_matching!(remain, updated.elem[i], updated.omni);
                }
            }

            if remain > 0 {
                consume!(remain, updated.omni);
            }

            if remain > 0 {
                return None;
            }
        }

        if unaligned > tot {
            return None;
        }

        // Pay unaligned costs
        if unaligned > 0 {
            let mut remain = unaligned;
            for e in updated.elem_order(ep) {
                let i = e.to_index();
                consume!(remain, updated.elem[i]);
            }
            consume!(remain, updated.omni);

            if remain > 0 {
                return None;
            }
        }

        Some(updated)
    }

    pub fn select_for_elemental_tuning(&self, ep: &ElementPriority) -> Option<Element> {
        for e in self.elem_order(ep) {
            let i = e.to_index();
            if self.elem[i] > 0 && !(ep.active_elem.is_some() && ep.active_elem == Some(e)) {
                return Some(e);
            }
        }
        None
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.omni == 0 && self.elem.iter().copied().all(|e| e == 0)
    }

    pub(crate) fn try_collect(&self, count: u8) -> Option<(u8, Self)> {
        let mut counter = DiceCounter::default();
        let mut taken = 0;
        for elem in Element::VALUES {
            let v = self[Dice::Elem(elem)];
            if taken >= count {
                break;
            }

            if v == 0 {
                continue;
            }
            counter.add_single(Dice::Elem(elem), 1);
            taken += 1
        }

        if taken < count && self[Dice::Omni] > 0 {
            counter.add_single(Dice::Omni, 1);
            taken += 1
        }

        if taken == 0 {
            None
        } else {
            Some((taken, counter))
        }
    }
}

impl Index<Dice> for DiceCounter {
    type Output = u8;

    #[inline]
    fn index(&self, index: Dice) -> &Self::Output {
        match index {
            Dice::Omni => &self.omni,
            Dice::Elem(e) => &self.elem[e.to_index()],
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::elem_set;

    use super::*;

    const fn cost(elem_cost: Option<(Element, u8)>, unaligned_cost: u8, aligned_cost: u8) -> Cost {
        Cost {
            elem_cost,
            unaligned_cost,
            aligned_cost,
            energy_cost: 0,
        }
    }

    #[test]
    fn test_from_tally() {
        assert_eq!(DiceCounter::default(), DiceCounter::new(&vec![]));
        assert_eq!(
            DiceCounter {
                omni: 2,
                ..DiceCounter::default()
            },
            DiceCounter::new(&vec![(Dice::Omni, 2)])
        );
        assert_eq!(
            DiceCounter {
                omni: 2,
                elem: [3, 2, 0, 0, 0, 1, 0]
            },
            DiceCounter::new(&vec![
                (Dice::Elem(Element::Pyro), 3),
                (Dice::Omni, 2),
                (Dice::Elem(Element::Geo), 1),
                (Dice::Elem(Element::Hydro), 2)
            ])
        );
    }

    #[test]
    fn test_try_pay_cost_unaligned() {
        assert_eq!(DiceCounter::new(&vec![]).try_pay_cost_short(&cost(None, 2, 0)), None);
        assert_eq!(
            DiceCounter::new(&vec![(Dice::Omni, 2)])
                .try_pay_cost_short(&cost(None, 2, 0))
                .unwrap(),
            DiceCounter::default()
        );
        assert_eq!(
            DiceCounter::new(&vec![(Dice::PYRO, 2)])
                .try_pay_cost_short(&cost(None, 2, 0))
                .unwrap(),
            DiceCounter::default()
        );
        assert_eq!(
            DiceCounter::new(&vec![(Dice::PYRO, 1), (Dice::CRYO, 1)])
                .try_pay_cost_short(&cost(None, 2, 0))
                .unwrap(),
            DiceCounter::default()
        );
    }

    #[test]
    fn test_try_pay_cost_unaligned_priority() {
        assert_eq!(DiceCounter::new(&vec![]).try_pay_cost_short(&cost(None, 2, 0)), None);
        assert_eq!(
            DiceCounter::new(&vec![(Dice::Omni, 2), (Dice::PYRO, 2)])
                .try_pay_cost_short(&cost(None, 2, 0))
                .unwrap(),
            DiceCounter::new(&vec![(Dice::Omni, 2)])
        );
        assert_eq!(
            DiceCounter::new(&vec![(Dice::PYRO, 2), (Dice::CRYO, 2)])
                .try_pay_cost(
                    &cost(None, 2, 0),
                    &ElementPriority::new(elem_set![], Some(Element::Pyro))
                )
                .unwrap(),
            DiceCounter::new(&vec![(Dice::PYRO, 2)])
        );
        assert_eq!(
            DiceCounter::new(&vec![(Dice::PYRO, 2), (Dice::CRYO, 2)])
                .try_pay_cost(&cost(None, 2, 0), &ElementPriority::new(elem_set![Element::Pyro], None))
                .unwrap(),
            DiceCounter::new(&vec![(Dice::PYRO, 2)])
        );
        assert_eq!(
            DiceCounter::new(&vec![
                (Dice::PYRO, 1),
                (Dice::CRYO, 1),
                (Dice::ELECTRO, 2),
                (Dice::ANEMO, 2)
            ])
            .try_pay_cost(
                &cost(None, 5, 0),
                &ElementPriority::new(elem_set![Element::Pyro, Element::Dendro], Some(Element::Cryo))
            )
            .unwrap(),
            DiceCounter::new(&vec![(Dice::CRYO, 1)])
        );
        assert_eq!(
            DiceCounter::new(&vec![
                (Dice::PYRO, 1),
                (Dice::CRYO, 1),
                (Dice::ELECTRO, 1),
                (Dice::ANEMO, 1)
            ])
            .try_pay_cost(
                &cost(None, 3, 0),
                &ElementPriority::new(elem_set![Element::Pyro, Element::Dendro], Some(Element::Cryo))
            )
            .unwrap(),
            DiceCounter::new(&vec![(Dice::CRYO, 1)])
        );
        assert_eq!(
            DiceCounter::new(&vec![(Dice::CRYO, 2), (Dice::Omni, 3)])
                .try_pay_cost(
                    &cost(None, 3, 0),
                    &ElementPriority::new(elem_set![], Some(Element::Cryo))
                )
                .unwrap(),
            DiceCounter::new(&vec![(Dice::Omni, 2)])
        );
    }

    #[test]
    fn test_try_pay_cost_matching() {
        assert_eq!(None, DiceCounter::new(&vec![]).try_pay_cost_short(&cost(None, 0, 2)));
        assert_eq!(
            None,
            DiceCounter::new(&vec![(Dice::PYRO, 1), (Dice::CRYO, 1)]).try_pay_cost_short(&cost(None, 0, 2))
        );
        assert_eq!(
            DiceCounter::default(),
            DiceCounter::new(&vec![(Dice::PYRO, 1), (Dice::Omni, 1)])
                .try_pay_cost_short(&cost(None, 0, 2))
                .unwrap()
        );
        assert_eq!(
            DiceCounter::default(),
            DiceCounter::new(&vec![(Dice::Omni, 2)])
                .try_pay_cost_short(&cost(None, 0, 2))
                .unwrap()
        );
        assert_eq!(
            DiceCounter::default(),
            DiceCounter::new(&vec![(Dice::Omni, 2)])
                .try_pay_cost_short(&cost(None, 2, 0))
                .unwrap()
        );
        assert_eq!(
            DiceCounter::default(),
            DiceCounter::new(&vec![(Dice::PYRO, 2)])
                .try_pay_cost_short(&cost(None, 2, 0))
                .unwrap()
        );
    }

    #[test]
    fn test_try_pay_cost_matching_priority() {
        assert_eq!(DiceCounter::new(&vec![]).try_pay_cost_short(&cost(None, 0, 2)), None);
        assert_eq!(
            DiceCounter::new(&vec![(Dice::PYRO, 2), (Dice::CRYO, 2)])
                .try_pay_cost(
                    &cost(None, 0, 2),
                    &ElementPriority::new(elem_set![], Some(Element::Pyro))
                )
                .unwrap(),
            DiceCounter::new(&vec![(Dice::PYRO, 2)])
        );
        assert_eq!(
            DiceCounter::new(&vec![(Dice::PYRO, 2), (Dice::CRYO, 2)])
                .try_pay_cost(&cost(None, 0, 2), &ElementPriority::new(elem_set![Element::Pyro], None))
                .unwrap(),
            DiceCounter::new(&vec![(Dice::PYRO, 2)])
        );
        assert_eq!(
            DiceCounter::new(&vec![
                (Dice::Omni, 3),
                (Dice::PYRO, 2),
                (Dice::DENDRO, 2),
                (Dice::CRYO, 2),
                (Dice::ANEMO, 2)
            ])
            .try_pay_cost(
                &cost(None, 0, 5),
                &ElementPriority::new(elem_set![Element::Pyro, Element::Dendro], Some(Element::Cryo))
            )
            .unwrap(),
            DiceCounter::new(&vec![(Dice::PYRO, 2), (Dice::DENDRO, 2), (Dice::CRYO, 2)])
        );
        assert_eq!(
            DiceCounter::new(&vec![
                (Dice::Omni, 2),
                (Dice::PYRO, 2),
                (Dice::DENDRO, 2),
                (Dice::CRYO, 3),
                (Dice::ANEMO, 2)
            ])
            .try_pay_cost(
                &cost(None, 0, 5),
                &ElementPriority::new(elem_set![Element::Pyro, Element::Dendro], Some(Element::Cryo))
            )
            .unwrap(),
            DiceCounter::new(&vec![(Dice::PYRO, 2), (Dice::DENDRO, 2), (Dice::ANEMO, 2)])
        );
    }

    #[test]
    fn test_try_pay_cost_omni_avoidance() {
        assert_eq!(
            DiceCounter::new(&vec![(Dice::Omni, 2), (Dice::DENDRO, 3), (Dice::PYRO, 5)])
                .try_pay_cost(
                    &cost(None, 5, 0),
                    &ElementPriority::new(elem_set![Element::Pyro], Some(Element::Cryo))
                )
                .unwrap(),
            DiceCounter::new(&vec![(Dice::Omni, 2), (Dice::PYRO, 3)])
        );

        assert_eq!(
            DiceCounter::new(&vec![(Dice::Omni, 2), (Dice::DENDRO, 3), (Dice::PYRO, 5)])
                .try_pay_cost(
                    &cost(None, 0, 5),
                    &ElementPriority::new(elem_set![Element::Pyro], Some(Element::Cryo))
                )
                .unwrap(),
            DiceCounter::new(&vec![(Dice::PYRO, 5)])
        );
    }

    #[test]
    fn test_try_collect_empty() {
        assert_eq!(None, DiceCounter::default().try_collect(0));
        assert_eq!(None, DiceCounter::default().try_collect(1));
    }

    #[test]
    fn test_try_collect_one() {
        for e in Element::VALUES {
            let d = DiceCounter::elem(e, 1);
            assert_eq!(Some((1, d)), d.try_collect(1));
            assert_eq!(Some((1, d)), DiceCounter::elem(e, 2).try_collect(1));
        }
        assert_eq!(Some((1, DiceCounter::omni(1))), DiceCounter::omni(1).try_collect(1));
        assert_eq!(Some((1, DiceCounter::omni(1))), DiceCounter::omni(2).try_collect(1));
    }

    #[test]
    fn test_try_collect_all() {
        let d = DiceCounter::new(&vec![(Dice::Omni, 2), (Dice::PYRO, 4), (Dice::CRYO, 1)]);
        let collected = DiceCounter::new(&vec![(Dice::Omni, 1), (Dice::PYRO, 1), (Dice::CRYO, 1)]);
        assert_eq!(Some((3, collected)), d.try_collect(3));
        assert_eq!(Some((3, collected)), d.try_collect(5));
    }

    #[test]
    fn test_try_collect_with_remainder() {
        let d = DiceCounter::new(&vec![
            (Dice::Omni, 2),
            (Dice::PYRO, 4),
            (Dice::CRYO, 3),
            (Dice::DENDRO, 1),
            (Dice::ELECTRO, 2),
        ]);
        let (count, collected) = d.try_collect(3).unwrap();
        assert_eq!(3, count);
        assert_eq!(3, collected.total());
        for e in Element::VALUES {
            assert!(collected[Dice::Elem(e)] <= 1);
        }
        assert!(collected[Dice::Omni] <= 1);
    }
}

// Properties of paying costs.
//
// Loosening transformations: Transformations on the dice or cost to make the cost easier to pay (outcome = pass/fail for paying the cost).
//  - Adding more dice preserves the outcome
//  - Converting dice into Omni preserves the outcome
//  - Updating the ElementPriority preserves the outcome
//
#[cfg(test)]
mod pay_cost_props {
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
            Self::unaligned(
                self.elem_cost.map(|(_, v)| v).unwrap_or_default() + self.unaligned_cost + self.aligned_cost,
            )
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

            #[test]
            fn same_number_of_omn_paid(d in arb_dice_counter(), cost in arb_tcg_cost_no_energy(), ep1 in arb_element_priority(), ep2 in arb_element_priority()) {
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
}
