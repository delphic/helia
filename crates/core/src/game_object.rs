use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

pub struct GameObject {
    components: HashMap<TypeId, Box<dyn Any>>,
}

impl GameObject {
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
        }
    }
    
    pub fn add_component<T: 'static>(&mut self, component: T) {
        self.components
            .insert(TypeId::of::<T>(), Box::new(component));
    }

    pub fn get_component<T: 'static>(&self) -> Option<&T> {
        let id = TypeId::of::<T>();
        if let Some(component) = self.components.get(&id) {
            return component.downcast_ref::<T>();
        }
        None
    }

    pub fn get_component_mut<T: 'static>(&mut self) -> Option<&mut T> {
        let id = TypeId::of::<T>();
        if let Some(component) = self.components.get_mut(&id) {
            return component.downcast_mut::<T>();
        }
        None
    }
}

