use std::sync::mpsc;

use super::*;
use crate::utils::Direction;

/// Message transmitters to the six neighboring processes
pub struct AdjacentProcs {
    xlo: Option<mpsc::Sender<AtomMessage>>,
    xhi: Option<mpsc::Sender<AtomMessage>>,
    ylo: Option<mpsc::Sender<AtomMessage>>,
    yhi: Option<mpsc::Sender<AtomMessage>>,
    zlo: Option<mpsc::Sender<AtomMessage>>,
    zhi: Option<mpsc::Sender<AtomMessage>>,
}
impl AdjacentProcs {
    pub fn new() -> Self {
        Self {
            xlo: None,
            xhi: None,
            ylo: None,
            yhi: None,
            zlo: None,
            zhi: None,
        }
    }
    pub fn as_vec(&self) -> Vec<&Option<mpsc::Sender<AtomMessage>>> {
        vec![
            &self.xlo, &self.xhi, &self.ylo, &self.yhi, &self.zlo, &self.zhi,
        ]
    }
    pub fn set(&mut self, direction: Direction, sender: mpsc::Sender<AtomMessage>) {
        match direction {
            Direction::Xlo => self.xlo = Some(sender),
            Direction::Xhi => self.xhi = Some(sender),
            Direction::Ylo => self.ylo = Some(sender),
            Direction::Yhi => self.yhi = Some(sender),
            Direction::Zlo => self.zlo = Some(sender),
            Direction::Zhi => self.zhi = Some(sender),
        };
    }
    pub fn xlo(&self) -> &Option<mpsc::Sender<AtomMessage>> {
        &self.xlo
    }
    pub fn xhi(&self) -> &Option<mpsc::Sender<AtomMessage>> {
        &self.xhi
    }
    pub fn ylo(&self) -> &Option<mpsc::Sender<AtomMessage>> {
        &self.ylo
    }
    pub fn yhi(&self) -> &Option<mpsc::Sender<AtomMessage>> {
        &self.yhi
    }
    pub fn zlo(&self) -> &Option<mpsc::Sender<AtomMessage>> {
        &self.zlo
    }
    pub fn zhi(&self) -> &Option<mpsc::Sender<AtomMessage>> {
        &self.zhi
    }
}
