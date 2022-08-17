use bevy_app::prelude::*;
use bevy_ecs::prelude::*;

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
    {
        let mut sets = std::collections::HashMap::new();
        add_state_exit::<Cleanup>(&mut sets);
        add_state_update(&mut sets);
        add_test_system(&mut sets);
        for (_, v) in sets.into_iter() {
            app.add_system_set(v);
        }
    };

    app.update();
}

fn add_state_exit<R: Component>(map: &mut std::collections::HashMap<(State, u8), SystemSet>) {
    let ss = map
        .remove(&(State::Test, 2u8))
        .unwrap_or_else(|| SystemSet::on_exit(State::Test));
    map.insert((State::Test, 2u8), ss.with_system(state_exit::<R>));
}

fn state_exit<R: Component>(mut c: Commands, q: Query<Entity, With<R>>) {
    c.entity(q.single()).despawn();
}

fn add_state_update(map: &mut std::collections::HashMap<(State, u8), SystemSet>) {
    let ss = map
        .remove(&(State::Test, 0u8))
        .unwrap_or_else(|| SystemSet::on_update(State::Test));
    map.insert((State::Test, 0u8), ss.with_system(state_update));
}
fn state_update(r: ResMut<TestRes>) {
    r.into_inner().0 += 1;
}
fn add_test_system(map: &mut std::collections::HashMap<(State, u8), SystemSet>) {
    let ss = map
        .remove(&(State::Test, 0u8))
        .unwrap_or_else(|| SystemSet::on_update(State::Test));
    map.insert(
        (State::Test, 0u8),
        ss.with_system(test_system.after(state_update)),
    );
}
fn test_system(r: ResMut<TestRes>) {
    let c = r.into_inner();

    c.0 += 1;
}
