/*
 * watchmymcserver: a hacked together solution to start and stop minecraft servers.
 *
 * copyright (c) Eason Qin, 2024
 *
 * This source code form is licensed under the Mozilla Public License v2.0 and
 * comes AS-IS with NO WARRANTY
 */

pub mod config;

mod manager;
mod server;

#[cfg(test)]
mod tests;

pub use manager::*;
pub use server::*;
