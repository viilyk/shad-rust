#![forbid(unsafe_code)]

pub use gc_derive::Scan;

use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    marker::PhantomData,
    ops::Deref,
    rc::{Rc, Weak},
};

////////////////////////////////////////////////////////////////////////////////

pub struct Gc<T> {
    weak: Weak<T>,
}

impl<T> Clone for Gc<T> {
    fn clone(&self) -> Self {
        Self {
            weak: self.weak.clone(),
        }
    }
}

impl<T> Gc<T> {
    pub fn borrow(&self) -> GcRef<'_, T> {
        let rc = self.weak.upgrade().unwrap();
        GcRef {
            rc,
            lifetime: PhantomData::<&'_ Gc<T>>,
        }
    }
    // pub fn addr(&self) -> usize {
    //     self.weak.as_ptr() as usize
    // }
}

pub struct GcRef<'a, T> {
    rc: Rc<T>,
    lifetime: PhantomData<&'a Gc<T>>,
}

impl<'a, T> Deref for GcRef<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.rc
    }
}

////////////////////////////////////////////////////////////////////////////////

pub trait Scan {
    fn get_ref_addr(&self, _c: &mut dyn FnMut(usize));
}

impl Scan for i32 {
    fn get_ref_addr(&self, _c: &mut dyn FnMut(usize)) {}
}

impl<T: Scan> Scan for Option<T> {
    fn get_ref_addr(&self, c: &mut dyn FnMut(usize)) {
        if let Some(x) = self {
            x.get_ref_addr(c);
        }
    }
}

impl<T: Scan> Scan for Vec<T> {
    fn get_ref_addr(&self, c: &mut dyn FnMut(usize)) {
        for x in self.iter() {
            x.get_ref_addr(c);
        }
    }
}

impl<T: Scan> Scan for RefCell<T> {
    fn get_ref_addr(&self, c: &mut dyn FnMut(usize)) {
        self.borrow().get_ref_addr(c);
    }
}

impl<T: Scan> Scan for Gc<T> {
    fn get_ref_addr(&self, c: &mut dyn FnMut(usize)) {
        c(self.weak.as_ptr() as usize)
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct Arena {
    objects: HashMap<usize, Rc<dyn Scan>>,
}

impl Arena {
    pub fn new() -> Self {
        Self {
            objects: HashMap::new(),
        }
    }

    pub fn allocation_count(&self) -> usize {
        self.objects.len()
    }

    pub fn alloc<T: Scan + 'static>(&mut self, obj: T) -> Gc<T> {
        let rc = Rc::new(obj);
        let weak = Rc::downgrade(&rc);
        let addr = Rc::as_ptr(&rc) as usize;
        self.objects.insert(addr, rc);
        Gc { weak }
    }

    pub fn sweep(&mut self) {
        let mut marked = HashSet::new();
        let mut count: HashMap<usize, usize> = HashMap::new();
        for (_, obj) in self.objects.iter() {
            obj.get_ref_addr(&mut |a| {
                count.entry(a).and_modify(|x| *x += 1).or_insert(1);
            });
        }
        for (addr, obj) in self.objects.iter() {
            if Rc::weak_count(obj) > *count.get(addr).unwrap_or(&0) {
                self.mark_all(*addr, &mut marked);
            }
        }
        let new = self.objects.clone();

        for (addr, obj) in new.into_iter().filter(|(a, _)| !marked.contains(a)) {
            drop(obj);
            self.objects.remove(&addr);
        }
    }

    fn mark_all(&self, root_addr: usize, marked: &mut HashSet<usize>) {
        if marked.insert(root_addr) {
            self.objects
                .get(&root_addr)
                .unwrap()
                .get_ref_addr(&mut |a| {
                    self.mark_all(a, marked);
                });
        }
    }
}

impl Default for Arena {
    fn default() -> Self {
        Self::new()
    }
}
