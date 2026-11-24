//! Program aplikasi Zevrigo Node untuk desktop (RENCANA_T16). Isi aplikasi ada di pustaka `zevrigo_node_app_lib`.

#![forbid(unsafe_code)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::ExitCode;

fn main() -> ExitCode {
    match zevrigo_node_app_lib::try_run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("zevrigo-node-app: {error}");
            ExitCode::FAILURE
        }
    }
}
