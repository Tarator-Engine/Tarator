use crate::prelude::*;

#[derive(Component, Clone)]
struct Position([u32; 2]);
impl CheckElement for Position {}

#[derive(Component, Clone)]
struct Rotation(u32);
impl CheckElement for Rotation {}

#[derive(Component, Clone)]
struct Label(String);
impl CheckElement for Label {}

#[derive(Component, Clone)]
struct Player;
impl CheckElement for Player {}

trait CheckElement: Component {}

#[derive(Callback)]
struct CheckComponents(usize);

impl<C: CheckElement> Callback<C> for CheckComponents {
    fn callback(&mut self, _: &C) {
        self.0 += 1;
    }
}

#[test]
fn callback() {
    let mut world = World::new();

    world.component_add_callback::<CheckComponents, Position>();
    world.component_add_callback::<CheckComponents, Rotation>();
    world.component_add_callback::<CheckComponents, Player>();
    world.component_add_callback::<CheckComponents, Label>();

    let entity = world.entity_create();
    world.entity_set(
        entity,
        (Position([0, 0]), Rotation(0), Player, Label("".into())),
    );

    let mut check = CheckComponents(0);
    world.entity_callback(entity, &mut check);
    assert!(check.0 == 4);
}

#[test]
fn component_query() {
    let mut world = World::new();

    fn init_entity<'a, T: CloneBundle>(world: &mut World, data: T) {
        for _ in 0..1 {
            let entity = world.entity_create();
            world.entity_set(entity, data.clone());
        }
    }

    init_entity(&mut world, Position([0, 0]));
    init_entity(&mut world, Rotation(0));
    init_entity(&mut world, Player);
    init_entity(&mut world, Label("Entity".to_owned()));
    init_entity(&mut world, (Position([0, 0]), Rotation(0)));
    init_entity(&mut world, (Position([0, 0]), Player));
    init_entity(&mut world, (Position([0, 0]), Label("Entity".to_owned())));
    init_entity(&mut world, (Position([0, 0]), Rotation(0), Player));
    init_entity(
        &mut world,
        (Position([0, 0]), Rotation(0), Label("Entity".to_owned())),
    );
    init_entity(
        &mut world,
        (
            Position([0, 0]),
            Rotation(0),
            Player,
            Label("Entity".to_owned()),
        ),
    );

    fn check_component<'a, T: Bundle>(world: &'a mut World, rec: usize) {
        let mut count = 0;
        world.component_query::<T>().for_each(|_| {
            count += 1;
        });
        assert!(count == rec, "{}", count);

        count = 0;
        world.component_query_mut::<T>().for_each(|_| {
            count += 1;
        });
        assert!(count == rec, "{}", count);
    }

    check_component::<Position>(&mut world, 7);
    check_component::<Rotation>(&mut world, 5);
    check_component::<Player>(&mut world, 4);
    check_component::<Label>(&mut world, 4);
    check_component::<(Position, Rotation)>(&mut world, 4);
    check_component::<(Position, Player)>(&mut world, 3);
    check_component::<(Position, Label)>(&mut world, 3);
    check_component::<(Position, Rotation, Player)>(&mut world, 2);
    check_component::<(Position, Rotation, Label)>(&mut world, 2);
    check_component::<(Position, Rotation, Player, Label)>(&mut world, 1);

    check_component::<(Rotation, Position)>(&mut world, 4);
    check_component::<(Player, Position)>(&mut world, 3);
    check_component::<(Label, Position)>(&mut world, 3);
    check_component::<(Player, Rotation, Position)>(&mut world, 2);
    check_component::<(Label, Rotation, Position)>(&mut world, 2);
    check_component::<(Label, Player, Rotation, Position)>(&mut world, 1);

    world.component_query::<Label>().for_each(|label| {
        assert!(label.0.as_str() == "Entity");
    });
}

#[test]
fn entity_unset() {
    let mut world = World::new();
    let entity = world.entity_create();
    world.entity_set(
        entity,
        (Position([0, 0]), Rotation(0), Player, Label("".into())),
    );
    assert!(world
        .entity_get::<(Position, Rotation, Player, Label)>(entity)
        .is_some());
    world.entity_unset::<(Position, Rotation, Label)>(entity);
    assert!(world.entity_get::<Position>(entity).is_none());
    assert!(world.entity_get::<Rotation>(entity).is_none());
    assert!(world.entity_get::<Player>(entity).is_some());
    assert!(world.entity_get::<Label>(entity).is_none());
}

