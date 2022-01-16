use super::memory_region::MemoryRegion;
use super::{Board, Error};
use crate::card::Card;
use crate::word::Word;
use borsh::{BorshDeserialize, BorshSerialize};

#[derive(Debug, Clone, Eq, PartialEq, BorshDeserialize, BorshSerialize)]
pub struct VerificationBoard {
    pub regions: Vec<OwnedMemoryRegion>,
    card_index: u32,
}

impl VerificationBoard {
    #[must_use]
    pub fn new() -> Self {
        Self {
            card_index: 0,
            regions: vec![],
        }
    }

    #[must_use]
    pub unsafe fn from_raw_parts(regions: Vec<OwnedMemoryRegion>, card_index: u32) -> Self {
        Self {
            regions,
            card_index,
        }
    }
}
impl Board for VerificationBoard {
    fn generate_card_id(&mut self) -> u32 {
        let id = self.card_index;
        self.card_index += 1;
        id
    }

    fn memory_region(
        &mut self,
        region_index: usize,
        player_id: u32,
    ) -> Result<&mut MemoryRegion, Error> {
        if region_index < self.regions.len() {
            let owned_region = &mut self.regions[region_index];
            if owned_region.owner() == player_id || owned_region.owner() == 0 {
                Ok(&mut owned_region.memory_region)
            } else {
                Err(Error::AccessViolation {
                    player_id,
                    region_index: region_index as u32,
                })
            }
        } else {
            Err(Error::NoSuchRegion {
                index: region_index as u32,
            })
        }
    }
    fn region_count(&self) -> usize {
        self.regions.len()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, BorshDeserialize, BorshSerialize)]
pub struct OwnedMemoryRegion {
    owner_id: u32,
    pub memory_region: MemoryRegion,
}

impl OwnedMemoryRegion {
    pub fn new(owner_id: u32) -> Self {
        Self {
            owner_id,
            memory_region: MemoryRegion::new(),
        }
    }
    pub fn with_data(owner_id: u32, cards: Vec<Card>, attrs: Vec<Word>) -> Self {
        Self {
            owner_id,
            memory_region: MemoryRegion::with_data(cards, attrs),
        }
    }

    #[must_use]
    pub fn owner(&self) -> u32 {
        self.owner_id
    }
}
