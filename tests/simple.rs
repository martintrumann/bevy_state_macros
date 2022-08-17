use bevy_app::prelude::*;
use bevy_ecs::prelude::*;

use bevy_state_macros::*;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
enum State {
    Test,
}

#[derive(Debug, PartialEq, Eq)]
struct TestRes(usize);

#[derive(Debug, Component)]
struct Cleanup;

#[test]
fn simple_app() {
    let mut app = App::new();

    app.add_state(State::Test).insert_resource(TestRes(0));

    add_systems!(app [
        state_exit::<Cleanup>,
        state_update,
        test_system,
    ]);

    app.update();

    assert_eq!(app.world.get_resource(), Some(&TestRes(2)))
}

#[on_exit(State::Test)]
fn state_exit<R: Component>(mut c: Commands, q: Query<Entity, With<R>>) {
    c.entity(q.single()).despawn();
}

#[on(State::Test)]
fn state_update(r: ResMut<TestRes>) {
    r.into_inner().0 += 1;
}

#[on(State::Test, .after(state_update))]
fn test_system(r: ResMut<TestRes>) {
    let c = r.into_inner();
    assert_eq!(c.0, 1);
    c.0 += 1;
}
