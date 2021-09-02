use crate::brick::{Action, BorshResult, Brick, Condition, Context, Value};
use crate::condition::BoolTerm;
use crate::value::ValueTerm;

use borsh::{BorshDeserialize, BorshSerialize};
use std::convert::TryInto;
use std::io::Write;
use std::rc::Rc;

use std::cmp;

impl BorshSerialize for Action {
    fn serialize<W: Write>(&self, writer: &mut W) -> BorshResult<()> {
        let action_code = 0u32.to_le_bytes();
        let code = self.get_code();
        writer.write_all(&action_code)?;
        writer.write_all(&code.to_le_bytes())?;
        let x = self.b_to_vec();
        writer.write_all(&x)?;
        Ok(())
    }
}

impl BorshDeserialize for Action {
    fn deserialize(buf: &mut &[u8]) -> std::result::Result<Self, std::io::Error> {
        let _action_code = u32::from_le_bytes(buf[..4].try_into().unwrap());
        *buf = &buf[4..];
        let code = u32::from_le_bytes(buf[..4].try_into().unwrap());
        *buf = &buf[4..];
        match code {
            0u32 => Ok(Box::new(Void::deserialize(buf)?)),
            1u32 => Ok(Box::new(Set::deserialize(buf)?)),
            2u32 => Ok(Box::new(Conditional::deserialize(buf)?)),
            3u32 => Ok(Box::new(Loop::deserialize(buf)?)),
            4u32 => Ok(Box::new(Card::deserialize(buf)?)),
            5u32 => Ok(Box::new(ShowMessage::deserialize(buf)?)),
            6u32 => Ok(Box::new(SetCtxVar::deserialize(buf)?)),
            100u32 => Ok(Box::new(MoveTo::deserialize(buf)?)),
            101u32 => Ok(Box::new(SetPlayerAttr::deserialize(buf)?)),
            102u32 => Ok(Box::new(AddPlayerAttr::deserialize(buf)?)),
            103u32 => Ok(Box::new(ApplyToPlace::deserialize(buf)?)),
            104u32 => Ok(Box::new(SubPlayerAttr::deserialize(buf)?)),
            _ => Ok(Box::new(Void {})),
        }
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Void {}

impl Brick<()> for Void {
    fn get_code(&self) -> u32 {
        return 0u32;
    }
    fn b_to_vec(&self) -> Vec<u8> {
        return self.try_to_vec().unwrap();
    }
    fn run(&mut self, _ctx: &mut Context) -> () {}
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Set {
    pub action1: Action,
    pub action2: Action,
}
impl Brick<()> for Set {
    fn get_code(&self) -> u32 {
        return 1u32;
    }
    fn b_to_vec(&self) -> Vec<u8> {
        return self.try_to_vec().unwrap();
    }
    fn run(&mut self, ctx: &mut Context) -> () {
        self.action1.run(ctx);
        self.action2.run(ctx);
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Conditional {
    pub condition: Condition,
    pub positive: Action,
    pub negative: Action,
}
impl Brick<()> for Conditional {
    fn get_code(&self) -> u32 {
        return 2u32;
    }
    fn b_to_vec(&self) -> Vec<u8> {
        return self.try_to_vec().unwrap();
    }
    fn run(&mut self, ctx: &mut Context) -> () {
        let cond = self.condition.run(ctx);
        if cond {
            self.positive.run(ctx)
        } else {
            self.negative.run(ctx)
        }
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Loop {
    pub iterations: Value,
    pub action: Action,
}
impl Brick<()> for Loop {
    fn get_code(&self) -> u32 {
        return 3u32;
    }
    fn b_to_vec(&self) -> Vec<u8> {
        return self.try_to_vec().unwrap();
    }
    fn run(&mut self, ctx: &mut Context) -> () {
        let iterations = self.iterations.run(ctx);
        for _ in 1..iterations {
            self.action.run(ctx);
        }
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Card {
    pub card_type: u32,
}
impl Brick<()> for Card {
    fn get_code(&self) -> u32 {
        return 4u32;
    }
    fn b_to_vec(&self) -> Vec<u8> {
        return self.try_to_vec().unwrap();
    }
    fn run(&mut self, ctx: &mut Context) -> () {
        let card_type = ctx.board.get_card_type_by_id(self.card_type);
        let mut action = card_type.unwrap().borrow_mut().get_action();
        action.run(ctx);
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct ShowMessage {
    pub message: Vec<u8>,
}
impl Brick<()> for ShowMessage {
    fn get_code(&self) -> u32 {
        return 5u32;
    }
    fn b_to_vec(&self) -> Vec<u8> {
        return self.try_to_vec().unwrap();
    }
    fn run(&mut self, ctx: &mut Context) -> () {
        let mut i = 0;
        let log = &mut ctx.board.log.borrow_mut();
        log.message_len = self.message.len().try_into().unwrap();
        log.message = [0; 128];
        for c in self.message.iter() {
            log.message[i] = *c;
            i += 1;
        }
        log.nonce += 1;
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct SetCtxVar {
    pub var_name: Vec<u8>,
    pub value: Value,
}
impl Brick<()> for SetCtxVar {
    fn get_code(&self) -> u32 {
        return 6u32;
    }
    fn b_to_vec(&self) -> Vec<u8> {
        return self.try_to_vec().unwrap();
    }
    fn run(&mut self, ctx: &mut Context) -> () {
        let value = self.value.run(ctx);
        ctx.vars.insert(self.var_name[..].to_vec(), value);
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct MoveTo {
    pub place: Value,
}
impl Brick<()> for MoveTo {
    fn get_code(&self) -> u32 {
        return 100u32;
    }
    fn b_to_vec(&self) -> Vec<u8> {
        return self.try_to_vec().unwrap();
    }
    fn run(&mut self, ctx: &mut Context) -> () {
        let place = self.place.run(ctx);
        let mut card = ctx.object.borrow_mut();
        card.place = place.try_into().unwrap();
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct SetPlayerAttr {
    pub attr_index: u32,
    pub player_index: Value,
    pub attr_value: Value,
}
impl Brick<()> for SetPlayerAttr {
    fn get_code(&self) -> u32 {
        return 101u32;
    }
    fn b_to_vec(&self) -> Vec<u8> {
        return self.try_to_vec().unwrap();
    }
    fn run(&mut self, ctx: &mut Context) -> () {
        let player_index = self.player_index.run(ctx);
        let attr_value = self.attr_value.run(ctx);
        let player = ctx.board.get_player(player_index.try_into().unwrap());
        player.unwrap().borrow_mut().numeral_attrs[self.attr_index as usize] = attr_value;
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct AddPlayerAttr {
    pub attr_index: u32,
    pub player_index: Value,
    pub attr_value: Value,
}
impl Brick<()> for AddPlayerAttr {
    fn get_code(&self) -> u32 {
        return 102u32;
    }
    fn b_to_vec(&self) -> Vec<u8> {
        return self.try_to_vec().unwrap();
    }
    fn run(&mut self, ctx: &mut Context) -> () {
        let player_index = self.player_index.run(ctx);
        let attr_value = self.attr_value.run(ctx);
        let player = ctx.board.get_player(player_index.try_into().unwrap());
        player.unwrap().borrow_mut().numeral_attrs[self.attr_index as usize] += attr_value;
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct SubPlayerAttr {
    pub attr_index: u32,
    pub player_index: Value,
    pub attr_value: Value,
}
impl Brick<()> for SubPlayerAttr {
    fn get_code(&self) -> u32 {
        return 104u32;
    }
    fn b_to_vec(&self) -> Vec<u8> {
        return self.try_to_vec().unwrap();
    }
    fn run(&mut self, ctx: &mut Context) -> () {
        let player_index = self.player_index.run(ctx);
        let attr_value = self.attr_value.run(ctx);
        let player = ctx.board.get_player(player_index.try_into().unwrap());
        player.unwrap().borrow_mut().numeral_attrs[self.attr_index as usize] -= attr_value;
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct ApplyToPlace {
    pub place: Value,
    pub action: Action,
    pub limit: Value,
}
impl Brick<()> for ApplyToPlace {
    fn get_code(&self) -> u32 {
        return 103u32;
    }
    fn b_to_vec(&self) -> Vec<u8> {
        return self.try_to_vec().unwrap();
    }
    fn run(&mut self, ctx: &mut Context) -> () {
        // JUST TOO MUCH
        let place = self.place.run(ctx);
        let mut limit = self.limit.run(ctx);
        let mut cards = ctx.board.get_cards_by_place(place.try_into().unwrap());
        ctx.board.rand.borrow_mut().shuffle(&mut cards); //
        if limit == 0 {
            limit = cards.len().try_into().unwrap();
        }
        let old_object = Rc::clone(&ctx.object);
        limit = cmp::min(limit, cards.len().try_into().unwrap());
        for _ in 0..limit {
            let new_object = Rc::clone(&cards.pop().unwrap());
            ctx.object = new_object;
            self.action.run(ctx);
        }
        ctx.object = old_object;
    }
}
