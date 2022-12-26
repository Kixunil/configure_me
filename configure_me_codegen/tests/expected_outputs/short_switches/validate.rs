            let d = self.d;
            let e = self.e;

            Ok(super::Config {
                d,
                e,
                a: self.a.unwrap_or(false),
                b: self.b.unwrap_or(false),
                c: self.c.unwrap_or(0),
                foo_bar: self.foo_bar.unwrap_or(false),
            })
