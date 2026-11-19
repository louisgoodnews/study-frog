#![allow(dead_code, clippy::unwrap_used, clippy::expect_used)] // shared test helpers
//! Helpers shared by the Universal Import Hub UI test files.

use std::fs;
use std::path::{Path, PathBuf};

use eframe::egui::DroppedFile;
use egui_kittest::Harness;
use studyfrog_ui::exec::Worker;
use studyfrog_ui::import_hub::HubMsg;
use studyfrog_ui::library_ui::LibraryMsg;
use studyfrog_ui::msg::Msg;
use studyfrog_ui::transfer_ui::TransferMsg;

use crate::common::flow::send;
use crate::common::run_to_idle;
use crate::common::views::App;

#[derive(Debug)]
pub struct TestDrop(pub PathBuf);
impl DroppedFile for TestDrop {
    fn path(&self) -> &Path {
        &self.0
    }
    fn bytes(&self) -> Result<Vec<u8>, String> {
        fs::read(&self.0).map_err(|e| e.to_string())
    }
}

pub fn hub(m: HubMsg) -> Msg {
    xfer(TransferMsg::Hub(Box::new(m)))
}

pub fn xfer(m: TransferMsg) -> Msg {
    Msg::Library(LibraryMsg::Transfer(m))
}

pub fn flush(h: &mut Harness<'_, App>, w: &mut Worker) {
    let todo = std::mem::take(&mut h.state_mut().1);
    run_to_idle(&mut h.state_mut().0, w, todo);
    h.run_steps(4);
}

pub fn send_ui(h: &mut Harness<'_, App>, w: &mut Worker, msg: Msg) {
    send(&mut h.state_mut().0, w, msg);
    h.run_steps(2);
}
