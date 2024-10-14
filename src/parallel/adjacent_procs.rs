use std::sync::mpsc;

use super::AtomInfo;
use crate::Direction;

/// Message transmitters to the six neighboring processes
pub struct AdjacentProcs {
    xlo: Option<mpsc::Sender<AtomInfo>>,
    xhi: Option<mpsc::Sender<AtomInfo>>,
    ylo: Option<mpsc::Sender<AtomInfo>>,
    yhi: Option<mpsc::Sender<AtomInfo>>,
    zlo: Option<mpsc::Sender<AtomInfo>>,
    zhi: Option<mpsc::Sender<AtomInfo>>,
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
    pub fn as_vec(&self) -> Vec<&Option<mpsc::Sender<AtomInfo>>> {
        vec![
            &self.xlo, &self.xhi, &self.ylo, &self.yhi, &self.zlo, &self.zhi,
        ]
    }
    pub fn set(&mut self, direction: Direction, sender: mpsc::Sender<AtomInfo>) {
        match direction {
            Direction::Xlo => self.xlo = Some(sender),
            Direction::Xhi => self.xhi = Some(sender),
            Direction::Ylo => self.ylo = Some(sender),
            Direction::Yhi => self.yhi = Some(sender),
            Direction::Zlo => self.zlo = Some(sender),
            Direction::Zhi => self.zhi = Some(sender),
        };
    }
    pub fn xlo(&self) -> &Option<mpsc::Sender<AtomInfo>> {
        &self.xlo
    }
    pub fn xhi(&self) -> &Option<mpsc::Sender<AtomInfo>> {
        &self.xhi
    }
    pub fn ylo(&self) -> &Option<mpsc::Sender<AtomInfo>> {
        &self.ylo
    }
    pub fn yhi(&self) -> &Option<mpsc::Sender<AtomInfo>> {
        &self.yhi
    }
    pub fn zlo(&self) -> &Option<mpsc::Sender<AtomInfo>> {
        &self.zlo
    }
    pub fn zhi(&self) -> &Option<mpsc::Sender<AtomInfo>> {
        &self.zhi
    }
}
