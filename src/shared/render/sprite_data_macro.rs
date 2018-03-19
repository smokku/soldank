macro_rules! sprites {
    (
        $($enm:ident {
            $($id:ident = $file:tt)*
        })+
    ) => {
        pub trait SpriteData where Self: ::std::marker::Sized {
            fn id(&self) -> i32;
            fn group(&self) -> SpriteGroup;
            fn filename(&self) -> &'static str;
            fn values() -> &'static [Self];
        }

        #[derive(Debug, Copy, Clone)]
        pub enum SpriteGroup {
            $($enm,)+
        }

        impl SpriteGroup {
            pub fn id(&self) -> i32 { *self as i32 }
            pub fn values() -> &'static [SpriteGroup] {
                static VALUES: &[SpriteGroup] = &[$(SpriteGroup::$enm,)+];
                VALUES
            }
        }

        $(
            #[derive(Debug, Copy, Clone)]
            pub enum $enm {
                $($id,)+
            }

            impl SpriteData for $enm {
                fn id(&self) -> i32 { *self as i32 }
                fn group(&self) -> SpriteGroup { SpriteGroup::$enm }
                fn filename(&self) -> &'static str {
                    match *self { $($enm::$id => $file,)+ }
                }
                fn values() -> &'static [$enm] {
                    static VALUES: &[$enm] = &[$($enm::$id,)+];
                    VALUES
                }
            }

            impl ::std::convert::From<i32> for $enm {
                fn from(id: i32) -> $enm {
                    match $enm::values().get(id as usize) {
                        Some(&v) => v,
                        _ => panic!("Invalid sprite identifier."),
                    }
                }
            }

            impl ::std::ops::Add<i32> for $enm {
                type Output = $enm;
                fn add(self, x: i32) -> $enm { $enm::from(self.id() + x) }
            }

            impl ::std::ops::Sub<i32> for $enm {
                type Output = $enm;
                fn sub(self, x: i32) -> $enm { $enm::from(self.id() - x) }
            }
        )+
    }
}
