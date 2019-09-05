            let d = self.d;
            let e = self.e;

            Ok(super::Config {
                d: d.map(Into::into),
                e: e.map(Into::into),
                a: self.a.unwrap_or(false),
                b: self.b.unwrap_or(false),
                c: self.c.unwrap_or(0),
            })
