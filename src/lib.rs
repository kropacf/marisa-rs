mod utils {
    use std::ffi::CString;

    #[inline(always)]
    pub fn to_raw(key: &str) -> (*mut i8, usize) {
        let cstr = CString::new(key).expect("CString new failed");
        let bytes = cstr.as_bytes_with_nul();
        let size = bytes.len();
        (cstr.into_raw(), size)
    }
}

pub mod marisa {
    use std::{
        ffi::{CStr, CString},
        str::Utf8Error,
    };

    use ffi::{
        marisa_Key, marisa_Key_Union, marisa_Keyset, marisa_Keyset_KEY_BLOCK_SIZE, marisa_Trie,
    };
    pub use marisa_sys as ffi;

    use crate::utils;

    #[derive(Clone)]
    pub struct Key {
        key: marisa_Key,
        drop: bool,
    }

    impl Default for Key {
        fn default() -> Self {
            Self {
                key: marisa_Key {
                    ptr_: std::ptr::null(),
                    length_: 0,
                    union_: marisa_Key_Union { id: 0 },
                },
                drop: false,
            }
        }
    }

    impl Drop for Key {
        fn drop(&mut self) {
            if !self.key.ptr_.is_null() && self.drop {
                let str = unsafe { CString::from_raw(self.key.ptr_ as *mut i8) };
                drop(str);
            }
        }
    }

    impl Key {
        pub fn new(key: &str) -> Key {
            let (ptr, size) = utils::to_raw(key);

            Key {
                key: marisa_Key {
                    ptr_: ptr,
                    length_: size as u32,
                    union_: marisa_Key_Union { id: 0 },
                },
                drop: true,
            }
        }

        pub fn set_id(&mut self, id: u32) {
            self.key.union_.id = id;
        }

        pub fn id(&self) -> u32 {
            unsafe { self.key.union_.id }
        }

        pub fn set_weight(&mut self, weight: f32) {
            self.key.union_.weight = weight;
        }

        pub fn weight(&self) -> f32 {
            unsafe { self.key.union_.weight }
        }

        pub fn set_str(&mut self, key: &str) {
            let (ptr, size) = utils::to_raw(key);

            self.key.ptr_ = ptr;
            self.key.length_ = size as u32;
            self.drop = true;
        }

        pub fn str(&self) -> Result<&str, Utf8Error> {
            let c = unsafe { CStr::from_ptr(self.key.ptr_ as *const i8) };
            let key = c.to_str()?;
            Ok(key)
        }

        pub fn ptr(&self) -> *const i8 {
            self.key.ptr_
        }

        pub fn length(&self) -> u32 {
            return self.key.length_;
        }

        pub fn from(existing: marisa_Key) -> Key {
            Self {
                key: existing,
                drop: false,
            }
        }
    }

    #[derive(Debug)]
    pub struct Keyset {
        keyset: marisa_Keyset,
    }

    impl Default for Keyset {
        fn default() -> Self {
            Self {
                keyset: unsafe { marisa_Keyset::new() },
            }
        }
    }

    impl Drop for Keyset {
        fn drop(&mut self) {
            self.clear();
        }
    }

    impl Keyset {
        pub fn push(&mut self, key: &str, weight: Option<f32>) {
            let (ptr, size) = utils::to_raw(key);

            unsafe {
                self.keyset.push_back3(ptr, size, weight.unwrap_or(1.0));
            }
            let str = unsafe { CString::from_raw(ptr as *mut i8) };
            drop(str);
        }

        pub fn empty(&self) -> bool {
            self.keyset.size_ == 0
        }

        pub fn reset(&mut self) {
            unsafe {
                self.keyset.reset();
            }
        }

        pub fn clear(&mut self) {
            unsafe {
                self.keyset.clear();
            }
        }

        pub fn num_keys(&self) -> usize {
            self.keyset.size_
        }

        pub fn at(&self, index: usize) -> Key {
            let outer = unsafe {
                std::slice::from_raw_parts(self.keyset.key_blocks_.array_, self.keyset.size_)
            };
            let outer_index = index / marisa_Keyset_KEY_BLOCK_SIZE as usize;
            let inner_index = index % marisa_Keyset_KEY_BLOCK_SIZE as usize;

            let inner_array = &outer[outer_index];
            let inner =
                unsafe { std::slice::from_raw_parts(inner_array.array_, self.keyset.size_) };
            let out_key = inner[inner_index];
            Key::from(out_key)
        }
    }

    pub struct Trie {
        trie: marisa_Trie,
    }

    impl Default for Trie {
        fn default() -> Self {
            Self {
                trie: unsafe { marisa_Trie::new() },
            }
        }
    }

    impl Trie {
        pub fn build(&mut self, keyset: &mut Keyset) {
            unsafe {
                self.trie.build(&mut keyset.keyset, 0);
            }
        }

        pub fn num_tries(&self) -> usize {
            unsafe { self.trie.num_tries() }
        }

        pub fn num_keys(&self) -> usize {
            unsafe { self.trie.num_keys() }
        }

        pub fn num_nodes(&self) -> usize {
            unsafe { self.trie.num_nodes() }
        }

        pub fn clear(&mut self) {
            unsafe { self.trie.clear() }
        }

        pub fn save(&self, path: &std::path::Path) {
            let (path, _) = utils::to_raw(&path.to_string_lossy());
            unsafe {
                self.trie.save(path);
            }
        }
    }

    #[cfg(test)]
    mod tests {
        mod key_tests {
            use std::ffi::CStr;

            use crate::marisa::{Key, Keyset};

            #[test]
            fn create_key() {
                let _m = Key::default();
            }

            #[test]
            fn create_new_key() {
                let s = String::from("koko");
                let k = Key::new(&s);

                assert_eq!(k.str(), Ok("koko"));
            }

            #[test]
            fn from_existing() {
                let s = String::from("koko");
                let k = Key::new(&s);

                {
                    let existing = Key::from(k.key);
                    assert_eq!(existing.str(), Ok("koko"));
                }
                assert_eq!(k.str(), Ok("koko"));
            }

            #[test]
            fn set_id() {
                let mut k = Key::default();
                k.set_id(12);

                assert_eq!(unsafe { k.key.union_.id }, 12);
            }

            #[test]
            fn get_id() {
                let mut k = Key::default();
                k.key.union_.id = 134;

                assert_eq!(k.id(), 134);
            }

            #[test]
            fn set_str() {
                let mut k = Key::default();
                let text = "kockopes".to_owned();
                k.set_str(&text);

                let c = unsafe { CStr::from_ptr(k.key.ptr_ as *const i8) };
                let key = c.to_str();

                assert!(key.is_ok());

                assert_eq!(key.unwrap(), &text);
                assert_eq!(k.key.length_, (text.len() + 1) as u32);
            }

            #[test]
            fn get_str() {
                let mut k = Key::default();
                let text = "pes".to_owned();
                k.set_str(&text);

                if let Ok(res) = k.str() {
                    assert_eq!(res, text);
                } else {
                    panic!("str() failed")
                }
            }

            #[test]
            fn work_with_keyset() {
                let mut keyset = Keyset::default();

                keyset.push("fufi", Some(0.8));
                keyset.push("fi", Some(0.5));
                keyset.push("fu", None);

                assert_eq!(keyset.num_keys(), 3);
                assert_eq!(keyset.at(0).str().unwrap(), "fufi");
                assert_eq!(keyset.at(0).weight(), 0.8);
                assert_eq!(keyset.at(1).str().unwrap(), "fi");
                assert_eq!(keyset.at(1).weight(), 0.5);
                assert_eq!(keyset.at(2).str().unwrap(), "fu");
                assert_eq!(keyset.at(2).weight(), 1.0);
            }
        }
    }
}
