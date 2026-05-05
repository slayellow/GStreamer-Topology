#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    gstreamer_to_topology_lib::run();
}
