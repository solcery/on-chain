use super::{Board, Error, MemoryRegion};
use crate::card::Card;
use crate::word::Word;
use borsh::{BorshDeserialize, BorshSerialize};

#[derive(Debug, Clone, Eq, PartialEq, BorshDeserialize, BorshSerialize)]
pub struct PlayerBoard {
    pub regions: Vec<Option<MemoryRegion>>,
    card_index: u32,
    player_id: u32,
}

impl PlayerBoard {
    #[must_use]
    pub fn new(player_id: u32) -> Self {
        Self {
            card_index: 0,
            regions: vec![],
            player_id,
        }
    }

    #[must_use]
    pub unsafe fn from_raw_parts(
        regions: Vec<Option<MemoryRegion>>,
        card_index: u32,
        player_id: u32,
    ) -> Self {
        Self {
            regions,
            card_index,
            player_id,
        }
    }
}
impl Board for PlayerBoard {
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
        // TODO: Create struct for unowned memory region
        if region_index < self.regions.len() {
            match &mut self.regions[region_index] {
                Some(memory_region) => Ok(memory_region),
                None => Err(Error::AccessViolation {
                    player_id,
                    region_index: region_index as u32,
                }),
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
