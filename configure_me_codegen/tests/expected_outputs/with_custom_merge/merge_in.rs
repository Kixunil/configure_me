            if let Some(foo) = other.foo {
                if let Some(foo_old) = &mut self.foo {
                    (|a: &mut u32, b: u32| *a += b)(foo_old, foo);
                } else {
                    self.foo = Some(foo);
                }
            }
            if let Some(bar) = other.bar {
                if let Some(bar_old) = &mut self.bar {
                    (|a: &mut String, b: String| a.push_str(&b))(bar_old, bar);
                } else {
                    self.bar = Some(bar);
                }
            }
