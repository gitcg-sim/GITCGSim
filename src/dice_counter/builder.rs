use super::DiceCounter;

crate::with_updaters!(
    #[derive(Default, Clone)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    pub struct DiceCounterBuilder {
        pub omni: u8,
        pub pyro: u8,
        pub hydro: u8,
        pub cryo: u8,
        pub electro: u8,
        pub dendro: u8,
        pub geo: u8,
        pub anemo: u8,
    }
);

impl DiceCounterBuilder {
    #[inline]
    pub const fn build(self) -> DiceCounter {
        DiceCounter {
            omni: self.omni,
            elem: [
                self.pyro,
                self.hydro,
                self.cryo,
                self.electro,
                self.dendro,
                self.geo,
                self.anemo,
            ],
        }
    }
}

impl DiceCounter {
    #[inline]
    pub const fn into_builder(self) -> DiceCounterBuilder {
        DiceCounterBuilder {
            omni: self.omni,
            pyro: self.elem[0],
            hydro: self.elem[1],
            cryo: self.elem[2],
            electro: self.elem[3],
            dendro: self.elem[4],
            geo: self.elem[5],
            anemo: self.elem[6],
        }
    }
}

crate::impl_from_to_builder!(DiceCounter, DiceCounterBuilder);
