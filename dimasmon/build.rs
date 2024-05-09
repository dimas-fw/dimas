// Copyright © 2024 Stephan Kunz

//! Build file for UI

fn main() {
	slint_build::compile("ui/main.slint").expect("Build of UI failed");
}
