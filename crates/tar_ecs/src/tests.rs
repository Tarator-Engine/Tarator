use std::{alloc::Layout, mem};

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
    fn callback(&mut self, _: &mut C) {
        self.0 += 1;
    }
}

unsafe fn drop_fn<T>(data: *mut u8) {
    data.cast::<T>().drop_in_place()
}

unsafe fn cb_fn<T: Callback<U>, U: Component>(callback: *mut u8, component: *mut u8) {
    (*callback.cast::<T>()).callback(&mut *component.cast::<U>());
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
fn callback_raw() {
    unsafe {
        let mut world = World::new();
        let callback_id = world.callback_init_raw(CheckComponents::NAME);

        // Position
        let component_info = ComponentInfo::new(
            Layout::new::<Position>(),
            mem::needs_drop::<Position>().then_some(drop_fn::<Position>),
        );
        let component_id = world.component_init_raw(Position::NAME, component_info);
        world.component_add_callback_raw(
            component_id,
            callback_id,
            cb_fn::<CheckComponents, Position>,
        );

        // Rotation
        let component_info = ComponentInfo::new(
            Layout::new::<Rotation>(),
            mem::needs_drop::<Rotation>().then_some(drop_fn::<Rotation>),
        );
        let component_id = world.component_init_raw(Rotation::NAME, component_info);
        world.component_add_callback_raw(
            component_id,
            callback_id,
            cb_fn::<CheckComponents, Rotation>,
        );

        // Player
        let component_info = ComponentInfo::new(
            Layout::new::<Player>(),
            mem::needs_drop::<Player>().then_some(drop_fn::<Player>),
        );
        let component_id = world.component_init_raw(Player::NAME, component_info);
        world.component_add_callback_raw(
            component_id,
            callback_id,
            cb_fn::<CheckComponents, Player>,
        );

        // Label
        let component_info = ComponentInfo::new(
            Layout::new::<Label>(),
            mem::needs_drop::<Label>().then_some(drop_fn::<Label>),
        );
        let component_id = world.component_init_raw(Label::NAME, component_info);
        world.component_add_callback_raw(
            component_id,
            callback_id,
            cb_fn::<CheckComponents, Label>,
        );

        let entity = world.entity_create();
        world.entity_set_raw(
            entity,
            <(Position, Rotation, Player, Label)>::NAMES,
            &[
                (&mut mem::ManuallyDrop::new(Position([0, 0]))) as *mut _ as *mut u8,
                (&mut mem::ManuallyDrop::new(Rotation(0))) as *mut _ as *mut u8,
                (&mut mem::ManuallyDrop::new(Player)) as *mut _ as *mut u8,
                (&mut mem::ManuallyDrop::new(Label("".into()))) as *mut _ as *mut u8,
            ],
        );
        let mut check = CheckComponents(0);
        world.entity_callback_raw(
            entity,
            CheckComponents::NAME,
            &mut check as *mut _ as *mut u8,
        );
        assert!(check.0 == 4);
    }
}

#[test]
fn component_querier() {
    let mut world = World::new();

    for _ in 0..10 {
        let entity = world.entity_create();
        world.entity_set(entity, Position([99, 99]));
    }

    for _ in 0..10 {
        let entity = world.entity_create();
        world.entity_set(entity, (Position([99, 99]), Player));
    }

    for _ in 0..10 {
        let entity = world.entity_create();
        world.entity_set(entity, (Position([99, 99]), Rotation(0)));
    }

    unsafe {
        let querier = world.component_querier(Position::NAMES);
        let id = world.component_init::<Position>();

        let mut i = 0;
        for indexer in querier {
            let data = indexer.get(id).unwrap().cast::<Position>();
            assert!((*data).0 == [99, 99]);
            i += 1;
        }

        assert!(i == 30, "{i}");
    }
}

#[test]
fn component_query() {
    let mut world = World::new();

    fn init_entity<T: CloneBundle>(world: &mut World, data: T) {
        println!("{:?}", T::NAMES);

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

    fn check_component<T: Bundle>(world: &mut World, rec: usize) {
        println!("{:?}", T::NAMES);

        let mut count = 0;
        world.component_query::<T>(|_, _| {
            count += 1;
        });
        assert!(count == rec, "{} : {:?}", count, T::NAMES);

        world.component_query_mut::<T>(|_, _| {
            count += 1;
        });
        assert!(count == rec * 2, "{} : {:?}", count, T::NAMES);

        unsafe {
            world.component_query_raw(T::NAMES, |_, _| {
                count += 1;
            });
        }
        assert!(count == rec * 3, "{} : {:?}", count, T::NAMES);
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

    world.component_query::<Label>(|_, label| {
        assert!(label.0.as_str() == "Entity");
    });
}
