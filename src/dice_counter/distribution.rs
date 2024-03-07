use crate::std_subset::{cmp::min, Vec};

use rand::prelude::*;

use super::*;

/// Represents the probability distribution for rolling Elemental Dice given
/// total number of dice, number of rerolls, preferred elements and fixed dice.
///
/// The reroll strategy is:
///  - Reroll as many times as possible.
///  - Only non-preferred and non-fixed dice are rerolled.
///
/// This type is used for implementing smart dice selection and fully automated dice reroll.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DiceDistribution {
    /// Number of dice
    pub count: u8,
    /// Maximum number of rerolls
    pub rerolls: u8,
    /// Describes the elements to keep (i.e. will not be rerolled)
    pub priority: ElementPriority,
    /// A list of guaranteed starting Elemental Dice. Maximum 4 distinct elements.
    pub fixed: [(Element, u8); 4],
}

/// Dice determinization policy. Determines how unknown (own or opponent) dice are
/// determinized given `DiceDistribution`.
/// Determinization = deterministic approximation of hidden information or random processes.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DiceDeterminization {
    AllOmnis,
    Simplified { extra_omnis: u8 },
    Randomized,
}

impl DiceDeterminization {
    pub fn determinize<R: Rng>(self, rng: &mut R, dist: DiceDistribution) -> DiceCounter {
        match self {
            Self::AllOmnis => DiceCounter::omni(dist.count),
            Self::Simplified { extra_omnis } => DiceCounter::simplified_dice(dist, extra_omnis),
            Self::Randomized => DiceCounter::rand_with_reroll(rng, dist),
        }
    }
}

impl DiceDistribution {
    #[inline]
    pub fn new(count: u8, rerolls: u8, priority: ElementPriority, fixed_vec: SmallVec<[(Element, u8); 4]>) -> Self {
        let mut fixed: [(Element, u8); 4] = [
            (Element::Pyro, 0),
            (Element::Pyro, 0),
            (Element::Pyro, 0),
            (Element::Pyro, 0),
        ];
        for (i, v) in fixed_vec.iter().copied().take(4).enumerate() {
            fixed[i] = v;
        }

        Self {
            count,
            rerolls,
            priority,
            fixed,
        }
    }

    // Let X be the number of desired dice after rolling n Elemental Dice with no fixed dice, with r rerolls
    // p is the chance of a single dice roll rolling into the desired element (example: Omni/Pyro/Cryo = 3/8)
    // Each dice has up to r rerolls and can be treated independently.
    // X ~ Binomial(n, 1 - (1 - p)^(r + 1))
    // E[X] = n(1 - f^(p + 1))
    pub fn avg_with_reroll(n: u8, preferred: u8, rerolls: u8) -> f32 {
        let n = n as f32;
        let f = (n - preferred as f32) / n;
        n * (1.0 - f32::powi(f, (rerolls + 1) as i32))
    }

    /// Get the number of fixed Elemental Dice.
    ///
    /// The return value will never be larger than `self.count`
    #[inline]
    pub fn fixed_count(&self) -> u8 {
        min(self.fixed.iter().map(|(_, v)| *v).sum(), self.count)
    }

    /// Get the number of fixed Elemental Dice for a specific element.
    ///
    /// The return value will never be larger than `self.count`
    #[inline]
    pub fn fixed_count_for_elem(&self, elem: Element) -> u8 {
        let v = self
            .fixed
            .iter()
            .copied()
            .filter_map(|(e, v)| if e == elem { Some(v) } else { None })
            .sum();
        min(v, self.count)
    }

    /// Get the average number of desired dice given element priority and number of rerolls.
    pub fn avg_desired(&self) -> f32 {
        Self::avg_with_reroll(
            self.count - self.fixed_count(),
            self.priority.elems().len() as u8,
            self.rerolls,
        )
    }

    /// Update the distribution to guarantee `count` number of dice
    /// to be the specified element `elem`
    pub fn guarantee_elem(&mut self, elem: Element, count: u8) -> bool {
        for (e, c) in self.fixed.iter_mut() {
            if *c == 0 {
                *e = elem;
                *c = count;
                return true;
            }
            if *e == elem {
                *c += count;
                return true;
            }
        }
        false
    }
}

impl DiceCounter {
    pub fn simplified_dice(dist: DiceDistribution, extra_omnis: u8) -> DiceCounter {
        let free = dist.count.saturating_sub(dist.fixed_count());
        let avg = dist.avg_desired().ceil() as u8;
        let omni_count: u8 = avg + extra_omnis;
        let mut dice_counter = DiceCounter::omni(omni_count);
        if omni_count < free {
            if let Some(off_elem) = dist.priority.get_off_element() {
                dice_counter.add_single(Dice::Elem(off_elem), free - omni_count);
            }
        }

        for (e, c) in dist.fixed {
            dice_counter.add_single(Dice::Elem(e), c);
        }

        dice_counter
    }

    pub fn rand_with_reroll<R: RngCore>(
        rng: &mut R,
        DiceDistribution {
            count,
            rerolls,
            priority,
            fixed,
        }: DiceDistribution,
    ) -> DiceCounter {
        let mut gen = move || {
            let n = rng.gen_range(0..=7);
            if n == 7 {
                Dice::Omni
            } else {
                Dice::Elem(Element::VALUES[n])
            }
        };

        let f: u8 = fixed.iter().copied().map(|(_, c)| c).sum();
        let rand_count = count.saturating_sub(f);
        let mut es = Vec::with_capacity(rand_count as usize);

        for _ in 0..rand_count {
            es.push(gen())
        }

        let desired_elems = priority.elems();
        for _ in 0..rerolls {
            for d in es.iter_mut() {
                let Dice::Elem(e) = *d else { continue };

                if desired_elems.contains(e) {
                    continue;
                }

                *d = gen();
            }
        }

        let mut c = Self::default();
        for d in es {
            c.add_single(d, 1);
        }

        for (e, count) in fixed {
            c.add_single(Dice::Elem(e), count);
        }
        c
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::elem_set;
    use smallvec::smallvec;

    #[test]
    fn test_rand_with_reroll_all_fixed_dice() {
        let mut r = SmallRng::seed_from_u64(10);
        assert_eq!(
            DiceCounter::elem(Element::Dendro, 4),
            DiceCounter::rand_with_reroll(
                &mut r,
                DiceDistribution::new(4, 0, Default::default(), smallvec![(Element::Dendro, 4)])
            )
        );

        let d = DiceCounter::rand_with_reroll(
            &mut r,
            DiceDistribution::new(
                6,
                0,
                Default::default(),
                smallvec![(Element::Dendro, 2), (Element::Cryo, 1), (Element::Electro, 3)],
            ),
        );
        assert_eq!(2, d[Dice::Elem(Element::Dendro)]);
        assert_eq!(1, d[Dice::Elem(Element::Cryo)]);
        assert_eq!(3, d[Dice::Elem(Element::Electro)]);
    }

    #[test]
    fn test_rand_with_reroll_fixed_dice_zero_rerolls() {
        let mut r = SmallRng::seed_from_u64(10);
        for _ in 0..1000 {
            let c = DiceCounter::rand_with_reroll(
                &mut r,
                DiceDistribution::new(
                    8,
                    0,
                    Default::default(),
                    smallvec![(Element::Pyro, 2), (Element::Electro, 1)],
                ),
            );
            assert!(c[Dice::Elem(Element::Pyro)] >= 2);
            assert!(c[Dice::Elem(Element::Electro)] >= 1);
        }
    }

    #[test]
    fn test_rand_with_reroll_fixed_dice_with_rerolls() {
        let mut r = SmallRng::seed_from_u64(10);
        for _ in 0..1000 {
            let c = DiceCounter::rand_with_reroll(
                &mut r,
                DiceDistribution::new(
                    8,
                    3,
                    Default::default(),
                    smallvec![(Element::Pyro, 2), (Element::Electro, 1)],
                ),
            );
            assert!(c[Dice::Elem(Element::Pyro)] >= 2);
            assert!(c[Dice::Elem(Element::Electro)] >= 1);
        }
    }

    fn ci95_proportion(avg: f32, n: u8, samples: u32) -> f32 {
        1.96 * f32::sqrt(avg * ((n as f32) - avg) / (samples as f32))
    }

    fn test_rand_with_reroll(r: &mut SmallRng, samples: u32, priority: ElementPriority) {
        for rerolls in 0..6 {
            let mut count = 0u32;
            for _ in 0..samples {
                let c =
                    DiceCounter::rand_with_reroll(r, DiceDistribution::new(8, rerolls, priority, Default::default()));
                count += c[Dice::Omni] as u32;
                for e in priority.elems() {
                    count += c[Dice::Elem(e)] as u32;
                }
            }
            let avg = (count as f32) / (samples as f32);
            let predicted_avg = DiceDistribution::avg_with_reroll(8, (priority.elems().len() as u8) + 1, rerolls);
            let ci95 = ci95_proportion(avg, 8, samples);
            println!("{rerolls}: actual={avg:.4} \u{b1} {ci95:.4}, predicted={predicted_avg:.4}");
            assert!((avg - ci95..avg + ci95).contains(&predicted_avg));
        }
    }

    const SAMPLES: u32 = 5000;
    #[test]
    fn test_rand_with_reroll_omni_dice_count() {
        let mut r = SmallRng::seed_from_u64(10);
        test_rand_with_reroll(&mut r, SAMPLES, Default::default());
    }

    #[test]
    fn test_rand_with_reroll_omni_plus_1_preferred() {
        let mut r = SmallRng::seed_from_u64(10);
        test_rand_with_reroll(
            &mut r,
            SAMPLES,
            ElementPriority::new(Default::default(), Some(Element::Pyro)),
        );
    }

    #[test]
    fn test_rand_with_reroll_omni_plus_2_preferred() {
        let mut r = SmallRng::seed_from_u64(10);
        test_rand_with_reroll(
            &mut r,
            SAMPLES,
            ElementPriority::new(elem_set![Element::Pyro, Element::Hydro], Some(Element::Pyro)),
        );
    }

    #[test]
    fn test_rand_with_reroll_omni_plus_3_preferred() {
        let mut r = SmallRng::seed_from_u64(10);
        test_rand_with_reroll(
            &mut r,
            SAMPLES,
            ElementPriority::new(
                elem_set![Element::Pyro, Element::Hydro, Element::Dendro],
                Some(Element::Pyro),
            ),
        );
    }
}
