#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(unused_assignments)]
#![allow(clippy::upper_case_acronyms)]
#[allow(clippy::module_inception)]
pub mod adapter;
#[allow(clippy::module_inception)]
pub mod application;
pub mod audit;
pub mod blockchain;
#[allow(clippy::module_inception)]
pub mod config;
pub mod core;
pub mod crypto;
pub mod i18n;
#[allow(clippy::module_inception)]
pub mod infrastructure;
#[allow(clippy::module_inception)]
pub mod interface;
pub mod monitoring;
pub mod mvp;
pub mod network;
pub mod ops;
pub mod plugin;
pub mod security;
#[allow(clippy::module_inception)]
pub mod service;
pub mod storage;
pub mod tools;
