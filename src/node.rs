use std::fmt;
use std::ptr;

use hash_stream::BITS_IN_POINTER;
use map::BitMap;

pub struct Node<K, V> {
    map_key: MapKey<K>,
    base_value: BaseValue<K, V>,
}

impl<K: fmt::Debug, V> fmt::Debug for Node<K, V> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        use self::Either::*;
        unsafe {
            match self.inner() {
                Left((k, v)) => {
                    writeln!(fmt, "{:?}:{:?}", *k, v)?;
                },
                Right((map, base)) => {
                    writeln!(fmt, "{:b}:", map.get())?;
                    let mut ones = 0;
                    for i in 0..BITS_IN_POINTER {
                        if map.is_set(i) {
                            write!(fmt, "  {}->{:?}", i, &*base.offset(ones))?;
                            ones += 1;
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

impl<K, V> Drop for Node<K, V> {
    fn drop(&mut self) {
        unsafe {
            if let Some(value) = self.base_value.get_value() {
                Box::from_raw(value);
                Box::from_raw(self.map_key.key);
            } else {
                Box::from_raw(self.base_value.get_base().unwrap());
            }
        }
    }
}

impl<K, V> Node<K, V> {
    pub fn empty() -> Self {
        Node {
            map_key: MapKey {
                map: BitMap::empty()
            },
            base_value: BaseValue::new_base(ptr::null_mut())
        }
    }

    pub fn key_value(key: K, value: V) -> Self {
        Node {
            map_key: MapKey {
                key: Box::into_raw(Box::new(key))
            },
            base_value: BaseValue::new_value(Box::into_raw(Box::new(value)))
        }
    }

    pub fn inner(&self) -> Either<(*mut K, *mut V), (BitMap, *mut Node<K, V>)> {
        unsafe {
            use self::Either::*;
            if self.base_value.is_value() {
                Left((self.map_key.key, self.base_value.value))
            } else {
                Right((self.map_key.map, (self.base_value.repr & !1) as *mut Node<K, V>))
            }
        }
    }

    pub fn set_base(&mut self, base: *mut Node<K, V>) -> Option<*mut Node<K, V>> {
        if self.base_value.is_value() {
            None
        } else {
            unsafe {
                let prev = (self.base_value.repr & !1) as *mut Node<K, V>;
                self.base_value.base = base;
                self.base_value.repr |= 1;
                Some(prev)
            }
        }
    }

    pub fn get_map(&mut self) -> Option<&mut BitMap> {
        if self.base_value.is_value() {
            None
        } else {
            unsafe {
                Some(&mut self.map_key.map)
            }
        }
    }

    pub fn set_value(&mut self, value: *mut V) -> Option<*mut V> {
        if self.base_value.is_value() {
            unsafe {
                Some(ptr::replace((&mut self.base_value.value) as *mut *mut V, value))
            }
        } else {
            None
        }
    }
}

pub enum Either<L, R> {
    Left(L),
    Right(R),
}

pub union MapKey<K> {
    map: BitMap,
    key: *mut K,
}

pub union BaseValue<K, V> {
    base: *mut Node<K, V>,
    value: *mut V,
    repr: usize,
}

impl<K,V> BaseValue<K, V> {
    pub fn is_value(&self) -> bool {
        unsafe { self.repr & 1 == 0 }
    }

    pub fn new_base(base: *mut Node<K, V>) -> Self {
        let mut res = BaseValue {
            base
        };
        unsafe {
            res.repr |= 1;
        }
        res
    }

    pub fn new_value(value: *mut V) -> Self {
        BaseValue {
            value
        }
    }

    pub fn get_value(&self) -> Option<*mut V> {
        unsafe {
            if self.is_value() {
                Some(self.value)
            } else {
                None
            }
        }
    }

    pub fn get_base(&self) -> Option<*mut Node<K, V>> {
        unsafe {
            if self.is_value() {
                None
            } else {
                Some((self.repr & !1) as *mut Node<K, V>)
            }
        }
    }
}