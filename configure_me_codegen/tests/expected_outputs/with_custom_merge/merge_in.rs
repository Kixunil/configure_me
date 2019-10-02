            match (&mut self.foo, other.foo) {
                (None, None) => (),
                (None, Some(val)) => self.foo = Some(val),
                (Some(_), None) => (),
                (Some(val0), Some(val1)) => (|a: &mut u32, b: u32| *a += b)(val0, val1),
            }
