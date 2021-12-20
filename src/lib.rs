#![doc = include_str!("../README.md")]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "connect_four")]
pub mod connect_four;

#[cfg(feature = "minesweeper")]
pub mod minesweeper;

#[cfg(feature = "reversi")]
pub mod reversi;

#[cfg(feature = "tictactoe")]
pub mod tictactoe;

#[cfg(feature = "std")]
mod std_lib {
    pub(crate) use std::{
        cmp::Ordering,
        collections::VecDeque,
        convert::Infallible,
        ops::{Index, IndexMut},
        vec::Vec,
    };
}

#[cfg(not(feature = "std"))]
#[macro_use]
extern crate alloc;

#[cfg(not(feature = "std"))]
mod std_lib {
    pub(crate) use alloc::{collections::VecDeque, vec::Vec};
    pub(crate) use core::{
        cmp::Ordering,
        convert::Infallible,
        ops::{Index, IndexMut},
    };
}
