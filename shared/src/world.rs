use core::any::TypeId;
use std::{
    any::type_name,
    fmt::Debug,
    ops::{Deref, DerefMut},
};

use hecs::{Archetype, ColumnBatchBuilder, ColumnBatchType, Component, World as HecsWorld};

// With help from https://github.com/AngelOfSol/fg_engine_v2_work/blob/master/src/roster/world.rs

/// An opaque registry that holds data that helps a World clone itself.
#[derive(Clone, Default, Debug)]
pub struct CloneRegistry(Vec<CloneEntry>);

impl CloneRegistry {
    /// Registers `T` with the registry, enabling `T` to be cloned in any
    /// archetypes that contain it.
    pub fn register<T: Clone + Component>(mut self) -> Self {
        if !self.0.iter().any(|item| item.type_id == TypeId::of::<T>()) {
            self.0.push(register::<T>());
        }
        self
    }
}

#[derive(Clone)]
struct CloneEntry {
    type_id: TypeId,
    type_name: &'static str,
    add_type: fn(&mut ColumnBatchType) -> (),
    add_values: fn(&mut ColumnBatchBuilder, &Archetype) -> (),
}
fn register<T: Component + Clone>() -> CloneEntry {
    CloneEntry {
        type_id: TypeId::of::<T>(),
        type_name: type_name::<T>(),
        add_type: |batch_type| {
            batch_type.add::<T>();
        },
        add_values: |batch, arch| {
            let mut writer = batch
                .writer::<T>()
                .unwrap_or_else(|| panic!("Missing type from batch: {}", type_name::<T>()));
            for item in arch
                .get::<T>()
                .unwrap_or_else(|| panic!("Missing type from archetype: {}", type_name::<T>()))
                .iter()
            {
                if writer.push(item.clone()).is_err() {
                    panic!()
                }
            }
        },
    }
}

impl Debug for CloneEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "CloneEntry {{ type_id: {:?}, type_name: {} }}",
            self.type_id, self.type_name
        )
    }
}

#[derive(Default)]
pub struct World {
    inner: HecsWorld,
    clone_registry: CloneRegistry,
}

impl World {
    pub fn new(clone_registry: CloneRegistry) -> Self {
        Self {
            inner: HecsWorld::new(),
            clone_registry,
        }
    }
}

impl Deref for World {
    type Target = HecsWorld;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl DerefMut for World {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl Clone for World {
    fn clone(&self) -> Self {
        let mut new_world = Self::new(self.clone_registry.clone());

        for archetype in self.archetypes().filter(|item| !item.is_empty()) {
            assert!(archetype.component_types().all(|item| self
                .clone_registry
                .0
                .iter()
                .any(|register| register.type_id == item)));

            let mut types = ColumnBatchType::new();
            for entry in self
                .clone_registry
                .0
                .iter()
                .filter(|item| archetype.has_dynamic(item.type_id))
            {
                (entry.add_type)(&mut types);
            }
            let mut batch = types.into_batch(archetype.len());
            for entry in self
                .clone_registry
                .0
                .iter()
                .filter(|item| archetype.has_dynamic(item.type_id))
            {
                (entry.add_values)(&mut batch, archetype);
            }
            let entities: Box<[_]> = archetype
                .ids()
                .iter()
                .map(|id| unsafe { self.find_entity_from_id(*id) })
                .collect();
            new_world.spawn_column_batch_at(&entities, batch.build().unwrap());
        }

        new_world
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn clone() {
        let mut world = World::new(
            CloneRegistry::default()
                .register::<String>()
                .register::<u32>(),
        );

        world.spawn((4u32,));
        world.spawn((8u32, "test".to_string()));
        world.spawn(("test".to_string(),));

        let cloned = world.clone();

        assert_eq!(
            world.query::<&u32>().iter().count(),
            cloned.query::<&u32>().iter().count()
        );
        assert_eq!(
            world.query::<&String>().iter().count(),
            cloned.query::<&String>().iter().count()
        );

        for (left, right) in world
            .query::<&u32>()
            .iter()
            .zip(cloned.query::<&u32>().iter())
        {
            assert_eq!(left, right);
        }

        for (left, right) in world
            .query::<&String>()
            .iter()
            .zip(cloned.query::<&String>().iter())
        {
            assert_eq!(left, right);
        }
    }
}
