# Bevy State Macros
This crate allows to register bevy state systems with attribute macros.

```rust
use bevy::prelude::*;
use bevy_state_macros::*;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
enum AppState {
	Menu,
	Game,
}

#[derive(Debug, Component)]
struct Menu;

fn main() {
	let mut app = App::new();
	app.add_state(AppState::Menu);

	add_systems!(app [
		spawn_menu,
		handle_menu,
		#[on_exit(AppState::Menu)]
		cleanup::<Menu>,
	]);
}

#[on_enter(AppState::Menu)]
fn spawn_menu(mut c: Commands) {
	// Spawn the menu
}

#[on(AppState::Menu)]
fn handle_menu() {
	// handle the menu input
}

// Also works with generics.
fn cleanup<C: Component>(mut c: Commands, q_componts: Query<Entity, With<C>>) {
	// handle the menu input
}
```
