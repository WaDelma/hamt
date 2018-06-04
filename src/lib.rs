#[macro_use]
extern crate array_macro;

use std::mem;
use std::ptr;
use std::collections::hash_map::RandomState;
use std::hash::{Hash, BuildHasher};
use hash_stream::HashStream;
use free_list::FreeList;
use node::Node;
use node::Either::*;

mod hash_stream;
mod free_list;
mod node;
mod map;

pub struct Hamt<K, V, S = RandomState> {
    state: S,
    root: Node<K, V>,
    free_list: FreeList<Node<K, V>>,
}

impl<K, V> Hamt<K, V, RandomState> 
    where K: Hash + Eq,
{
    pub fn new() -> Hamt<K, V, RandomState> {
        Hamt {
            state: RandomState::new(),
            root: Node::empty(),
            free_list: FreeList::new(),
        }
    }
}

impl<K, V, S> Hamt<K, V, S>
    where K: Hash + Eq,
          S: BuildHasher + Default,
{
    fn key_hash(state: &S, key: &K) -> HashStream<S::Hasher> {
        let mut hasher = state.build_hasher();
        key.hash(&mut hasher);
        HashStream::new(hasher)
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        let mut stream = Self::key_hash(&self.state, key);
        let mut cur = &self.root;
        loop {
            break match cur.inner() {
                Left((k, v)) => if unsafe { &*k } == key {
                    Some(unsafe { &*v })
                } else {
                    None
                },
                Right((map, base)) => if map.is_empty() {
                    None
                } else {
                    let i = stream.next().unwrap() as usize;
                    if map.is_set(i) {
                        let n = map.set_bits_under(i) as isize;
                        cur = unsafe { &*base.offset(n) };
                        continue;
                    } else {
                        None
                    }
                }
            }
        }
    }

    pub fn insert(&mut self, mut key: K, mut value: V) -> Option<V> {
        let mut stream = Self::key_hash(&self.state, &key);
        let mut depth = 0;
        let mut cur = &mut self.root;
        loop {
            break match cur.inner() {
                Left((k, v)) => if unsafe { &*k } == &key {
                    let value = Box::into_raw(Box::new(value));
                    let old = cur.set_value(value).unwrap();
                    unsafe { Some(*Box::from_raw(old)) }
                } else {
                    let mut key_other = unsafe { ptr::read(k) };
                    let mut value_other = unsafe { ptr::read(v) };
                    let mut stream_other = Self::key_hash(&self.state, &key_other).skip(depth);

                    unsafe { ptr::write(cur, Node::empty()); }
                    loop {
                        let i = stream.next().unwrap() as usize;
                        let j = stream_other.next().unwrap() as usize;
                        if let Right((_, _)) = cur.inner() {
                            if i == j {
                                cur.get_map().map(|m| m.set(i));
                                let mut singleton = self.free_list.singleton(Node::empty());
                                let ptr = singleton.as_mut_ptr();
                                cur.set_base(ptr);
                                cur = unsafe { &mut *ptr };
                                mem::forget(singleton);
                            } else {
                                cur.get_map().map(|m| {
                                    m.set(i);
                                    m.set(j);
                                });
                                if j < i {
                                    mem::swap(&mut key, &mut key_other);
                                    mem::swap(&mut value, &mut value_other);
                                }
                                let mut pair = self.free_list.pair(
                                    Node::key_value(key, value),
                                    Node::key_value(key_other, value_other)
                                );
                                cur.set_base(pair.as_mut_ptr());
                                mem::forget(pair);
                                break;
                            }
                        } else {
                            unreachable!()
                        }
                    }
                    None
                },
                Right((map, base)) => {
                    let i = stream.next().unwrap() as usize;
                    depth += 1;
                    if map.is_empty() {
                        unsafe { ptr::write(cur, Node::key_value(key, value)); }
                        None
                    } else if map.is_set(i) {
                        let n = map.set_bits_under(i) as isize;
                        cur = unsafe { &mut *base.offset(n) };
                        continue
                    } else {
                        let bits = map.set_bits();
                        let pos = map.set_bits_under(i);
                        cur.get_map().map(|m| m.set(i));

                        let slice = unsafe { slice_from_raw(base, bits) };
                        let key_value = Node::key_value(key, value);

                        let mut slice = self.free_list.push(slice, key_value);
                        for i in (pos..(slice.len() - 1)).rev() {
                            slice.swap(i, i + 1);
                        }
                        cur.set_base(slice.as_mut_ptr());
                        mem::forget(slice);

                        None
                    }
                }
            }
        }
    }
}

unsafe fn slice_from_raw<T>(ptr: *mut T, len: usize) -> Box<[T]> {
    Vec::from_raw_parts(ptr, len, len).into_boxed_slice()
}

#[test]
fn insert_get() {
    let mut hamt = Hamt::new();
    for i in 0..5000 {
        hamt.insert(i, i);
        for a in 0..=i {
            assert_eq!(
                Some(&a),
                hamt.get(&a)
            );
        }
    }
}