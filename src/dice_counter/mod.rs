use core::cell::RefCell;

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

thread_local! {
    /// Memoization for ElementPriority::elem_order. Index by 8 possible values of `active_elem` (7 `Some(elem)` and `None`)
    /// and 128 possible values of `important_elems` (2^7)
    static ELEM_ORDER_CACHE: RefCell<[[Option<[Element; 7]>; 128]; 8]> = RefCell::new([[None; 128]; 8]);
}

/// Describes the preferred elements to keep for selecting Elemental Dice for
/// (1) cost payments and (2) automatic dice rerolls
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ElementPriority {
    /// Preferred elements (lower priority)
    important_elems: ElementSet,
    /// Element of the active character, if applicable (higher priority)
    active_elem: Option<Element>,
    elem_order: [Element; 7],
}

impl ConstDefault for ElementPriority {
    const DEFAULT: Self = Self {
        important_elems: enum_set![],
        active_elem: None,
        elem_order: Element::VALUES,
    };
}

impl Default for ElementPriority {
    fn default() -> Self {
        ConstDefault::DEFAULT
    }
}

impl ElementPriority {
    #[inline]
    pub fn new(mut important_elems: ElementSet, active_elem: Option<Element>) -> ElementPriority {
        if let Some(e) = active_elem {
            important_elems.insert(e);
        };
        let elem_order = Self::updated_elem_order(active_elem, important_elems);
        ElementPriority {
            important_elems,
            active_elem,
            elem_order,
        }
    }

    #[inline]
    pub fn elem_order(&self) -> [Element; 7] {
        self.elem_order
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
    pub fn off_element(&self) -> Option<Element> {
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

    fn updated_elem_order(active_elem: Option<Element>, important_elems: ElementSet) -> [Element; 7] {
        #[inline]
        fn option_elem_to_index(e: Option<Element>) -> usize {
            e.map_or(7, |e| e.to_index())
        }

        let compute = || {
            let mut elem_order = Element::VALUES;
            elem_order.sort_by_key(|&e| {
                if active_elem == Some(e) {
                    2
                } else if important_elems.contains(e) {
                    1
                } else {
                    0
                }
            });
            elem_order
        };

        ELEM_ORDER_CACHE.with_borrow_mut(|arr| {
            let entry = &mut arr[option_elem_to_index(active_elem)][important_elems.as_u8() as usize];
            if let Some(es) = entry {
                *es
            } else {
                let es = compute();
                *entry = Some(es);
                es
            }
        })
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

    /// Produce an ordering of elements to pay cost for given current Elemental
    /// Dice and and `ElementPriority`.
    ///
    /// The ordering is as follows:
    /// 1. Not prioritized, dice count below 3
    /// 2. Not prioritized, dice count 3 or above
    /// 3. Prioritized in preferred elements
    /// 4. Prioritized in active element
    fn elem_order(&self, ep: &ElementPriority) -> heapless::Vec<Element, 7> {
        let mut elems: heapless::Vec<Element, 7> = ep
            .elem_order()
            .iter()
            .copied()
            .filter(|&e| self.elem[e.to_index()] > 0)
            .collect();
        let in_ep = ep.elems();
        elems.sort_by_key(|&e| {
            if in_ep.contains(e) {
                2
            } else {
                let v = self.elem[e.to_index()];
                if v >= 3 {
                    1
                } else {
                    0
                }
            }
        });
        elems
    }

    #[cfg(any())]
    fn elem_order(&self, ep: &ElementPriority) -> heapless::Vec<Element, 7> {
        let mut elems: SmallVec<[Element; 7]> = Element::VALUES
            .iter()
            .copied()
            .filter(|&e| self.elem[e.to_index()] > 0)
            .collect();

        let ElementPriority {
            important_elems,
            active_elem,
            ..
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

#[cfg(test)]
mod pay_cost_props;
