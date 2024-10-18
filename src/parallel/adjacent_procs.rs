use std::sync::mpsc;

use super::message::Message;
use crate::Direction;

/// Message transmitters to the six neighboring processes
pub struct AdjacentProcs {
    xlo: Option<mpsc::Sender<Message>>,
    xhi: Option<mpsc::Sender<Message>>,
    ylo: Option<mpsc::Sender<Message>>,
    yhi: Option<mpsc::Sender<Message>>,
    zlo: Option<mpsc::Sender<Message>>,
    zhi: Option<mpsc::Sender<Message>>,
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
    pub fn as_vec(&self) -> Vec<&Option<mpsc::Sender<Message>>> {
        vec![
            &self.xlo, &self.xhi, &self.ylo, &self.yhi, &self.zlo, &self.zhi,
        ]
    }
    pub fn set(&mut self, direction: Direction, sender: mpsc::Sender<Message>) {
        match direction {
            Direction::Xlo => self.xlo = Some(sender),
            Direction::Xhi => self.xhi = Some(sender),
            Direction::Ylo => self.ylo = Some(sender),
            Direction::Yhi => self.yhi = Some(sender),
            Direction::Zlo => self.zlo = Some(sender),
            Direction::Zhi => self.zhi = Some(sender),
        };
    }
    pub fn xlo(&self) -> &Option<mpsc::Sender<Message>> {
        &self.xlo
    }
    pub fn xhi(&self) -> &Option<mpsc::Sender<Message>> {
        &self.xhi
    }
    pub fn ylo(&self) -> &Option<mpsc::Sender<Message>> {
        &self.ylo
    }
    pub fn yhi(&self) -> &Option<mpsc::Sender<Message>> {
        &self.yhi
    }
    pub fn zlo(&self) -> &Option<mpsc::Sender<Message>> {
        &self.zlo
    }
    pub fn zhi(&self) -> &Option<mpsc::Sender<Message>> {
        &self.zhi
    }
}
