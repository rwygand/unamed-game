use crate::actions::PlayerMovedEvent;
use bevy::prelude::*;

use crate::board::{components::Position, CurrentBoard};
use crate::pieces::components::{Health, Occupier};
use crate::player::Player;
use crate::vectors::Vector2Int;

use super::Action;

pub struct DamageAction(pub Entity, pub u32);
impl Action for DamageAction {
    fn execute(&self, world: &mut World) -> Result<Vec<Box<dyn Action>>, ()> {
        let Some(mut health) = world.get_mut::<Health>(self.0) else { return Err(()) };
        health.value = health.value.saturating_sub(self.1);
        if health.value == 0 {
            // the unit is killed
            world.despawn(self.0);
        }
        Ok(Vec::new())
    }
}

pub struct MeleeHitAction {
    pub attacker: Entity,
    pub target: Vector2Int,
    pub damage: u32,
}
impl Action for MeleeHitAction {
    fn execute(&self, world: &mut World) -> Result<Vec<Box<dyn Action>>, ()> {
        let attacker_position = world.get::<Position>(self.attacker).ok_or(())?;
        if attacker_position.v.manhattan(self.target) > 1 {
            return Err(());
        };
        let target_entities = world
            .query_filtered::<(Entity, &Position), With<Health>>()
            .iter(world)
            .filter(|(_, p)| p.v == self.target)
            .collect::<Vec<_>>();
        if target_entities.is_empty() {
            return Err(());
        };
        let result = target_entities
            .iter()
            .map(|e| Box::new(DamageAction(e.0, self.damage)) as Box<dyn Action>)
            .collect::<Vec<_>>();
        Ok(result)
    }
}

pub struct WalkAction(pub Entity, pub Vector2Int);
impl Action for WalkAction {
    fn execute(&self, world: &mut World) -> Result<Vec<Box<dyn Action>>, ()> {
        let board = world.get_resource::<CurrentBoard>().ok_or(())?;
        if !board.tiles.contains_key(&self.1) {
            return Err(());
        };
        if world
            .query_filtered::<&Position, With<Occupier>>()
            .iter(world)
            .any(|p| p.v == self.1)
        {
            // Can't move into occupier
            return Err(());
        };

        if world.get::<Player>(self.0).is_some() {
            // the entith that moved is the pplayer
            world.send_event(PlayerMovedEvent(self.1));
        }

        let mut position = world.get_mut::<Position>(self.0).ok_or(())?;
        position.v = self.1;
        Ok(Vec::new())
    }
}
