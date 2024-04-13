use crate::{
    data::ObjectId,
    error::*,
    object::{Object, Store},
    storage::StorageTransaction,
};

use std::{
    any::Any,
    cell::{Ref, RefCell, RefMut},
    collections::HashMap,
    marker::PhantomData,
    rc::Rc,
};

////////////////////////////////////////////////////////////////////////////////
type StatedObject = (Rc<RefCell<ObjectState>>, Rc<RefCell<dyn Store>>);

pub struct Transaction<'a> {
    inner: Box<dyn StorageTransaction + 'a>,
    cash: RefCell<HashMap<ObjectId, StatedObject>>,
}

impl<'a> Transaction<'a> {
    pub(crate) fn new(inner: Box<dyn StorageTransaction + 'a>) -> Self {
        Self {
            inner,
            cash: RefCell::new(HashMap::new()),
        }
    }

    pub fn create<T: Object>(&self, obj: T) -> Result<Tx<'_, T>> {
        let schema = T::SCHEMA;
        if !self.inner.table_exists(schema.table_name)? {
            self.inner.create_table(&schema)?;
        }
        let id = self.inner.insert_row(&schema, &obj.to_row())?;
        let obj_ref: Rc<RefCell<dyn Store>> = Rc::new(RefCell::new(obj));
        let state_ref: Rc<RefCell<ObjectState>> = Rc::new(RefCell::new(ObjectState::Clean));
        self.cash
            .borrow_mut()
            .insert(id, (Rc::clone(&state_ref), Rc::clone(&obj_ref)));
        Ok(Tx {
            lifetime: PhantomData::<&'_ T>,
            object: Rc::clone(&obj_ref),
            state: Rc::clone(&state_ref),
            id,
        })
    }

    pub fn get<T: Object>(&self, id: ObjectId) -> Result<Tx<'_, T>> {
        let schema = T::SCHEMA;
        if !self.inner.table_exists(schema.table_name)? {
            return Err(Error::NotFound(Box::new(NotFoundError {
                object_id: id,
                type_name: schema.object_name,
            })));
        }
        if let Some((st, rc)) = self.cash.borrow().get(&id) {
            if let ObjectState::Removed = *st.borrow() {
                return Err(Error::NotFound(Box::new(NotFoundError {
                    object_id: id,
                    type_name: schema.object_name,
                })));
            }
            return Ok(Tx {
                lifetime: PhantomData::<&'_ T>,
                object: rc.clone(),
                state: st.clone(),
                id,
            });
        }
        let row = self.inner.select_row(id, &schema)?;
        let obj = T::from_row(&row);
        let st: Rc<RefCell<ObjectState>> = Rc::new(RefCell::new(ObjectState::Clean));
        let rc = Rc::new(RefCell::new(obj));
        self.cash.borrow_mut().insert(id, (st.clone(), rc.clone()));
        Ok(Tx {
            lifetime: PhantomData::<&'_ T>,
            object: rc.clone(),
            state: st.clone(),
            id,
        })
    }

    pub fn commit(self) -> Result<()> {
        for (id, (state, obj)) in self.cash.borrow().iter() {
            let a = obj.borrow();
            let row = a.to_row();
            let b = obj.borrow();
            let schema = b.get_schema();
            match *state.borrow() {
                ObjectState::Clean => continue,
                ObjectState::Removed => self.inner.delete_row(*id, &schema)?,
                ObjectState::Modified => self.inner.update_row(*id, &schema, &row)?,
            }
        }
        self.inner.commit()?;
        Ok(())
    }

    pub fn rollback(self) -> Result<()> {
        self.inner.rollback()?;
        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy)]
pub enum ObjectState {
    Clean,
    Modified,
    Removed,
}

#[derive(Clone)]
pub struct Tx<'a, T> {
    lifetime: PhantomData<&'a T>,
    object: Rc<RefCell<dyn Store>>,
    state: Rc<RefCell<ObjectState>>,
    id: ObjectId,
}

impl<'a, T: Any> Tx<'a, T> {
    pub fn id(&self) -> ObjectId {
        self.id
    }

    pub fn state(&self) -> ObjectState {
        *self.state.borrow()
    }

    pub fn borrow(&self) -> Ref<'_, T> {
        if let ObjectState::Removed = *self.state.borrow() {
            panic!("cannot borrow a removed object");
        }
        Ref::map(self.object.borrow(), |r| r.as_any().downcast_ref().unwrap())
    }

    pub fn borrow_mut(&self) -> RefMut<'_, T> {
        if let ObjectState::Removed = *self.state.borrow() {
            panic!("cannot borrow a removed object");
        }
        *self.state.borrow_mut() = ObjectState::Modified;
        RefMut::map(self.object.borrow_mut(), |r| {
            r.as_any_mut().downcast_mut().unwrap()
        })
    }

    pub fn delete(self) {
        if self.object.try_borrow_mut().is_err() {
            panic!("cannot delete a borrowed object");
        }
        *self.state.borrow_mut() = ObjectState::Removed;
    }
}
