macro_rules! sprites {
    (
        $($enm:ident {
            $($id:ident = $file:tt)+
        })+
    ) => {
        pub trait SpriteData where Self: ::std::marker::Sized {
            fn id(&self) -> usize;
            fn group(&self) -> Group;
            fn filename(&self) -> &'static str;
            fn values() -> &'static [Self];
        }

        #[derive(Debug, Copy, Clone)]
        pub enum Group {
            $($enm,)+
        }

        impl Group {
            pub fn id(&self) -> usize { *self as usize }
            pub fn values() -> &'static [Group] {
                static VALUES: &[Group] = &[$(Group::$enm,)+];
                VALUES
            }
        }

        $(
            #[derive(Debug, Copy, Clone)]
            pub enum $enm {
                $($id,)+
            }

            impl SpriteData for $enm {
                fn id(&self) -> usize { *self as usize }
                fn group(&self) -> Group { Group::$enm }
                fn filename(&self) -> &'static str {
                    match *self { $($enm::$id => $file,)+ }
                }
                fn values() -> &'static [$enm] {
                    static VALUES: &[$enm] = &[$($enm::$id,)+];
                    VALUES
                }
            }

            impl ::std::convert::From<usize> for $enm {
                fn from(id: usize) -> $enm {
                    match $enm::values().get(id as usize) {
                        Some(&v) => v,
                        _ => panic!("Invalid sprite identifier."),
                    }
                }
            }

            impl ::std::ops::Add<usize> for $enm {
                type Output = $enm;
                fn add(self, x: usize) -> $enm { $enm::from(self.id() + x) }
            }

            impl ::std::ops::Sub<usize> for $enm {
                type Output = $enm;
                fn sub(self, x: usize) -> $enm { $enm::from(self.id() - x) }
            }
        )+
    }
}

macro_rules! gostek_parts_sprite {
    ( None ) => ( GostekSprite::None );
    ( $group:ident::$id:ident ) => ( GostekSprite::$group($group::$id) );
}

macro_rules! gostek_parts {
    (
        $(
            $id:ident =
                Sprite($($sprite:tt)+),
                Point($p1:expr, $p2:expr),
                Center($cx:expr, $cy:expr),
                Show($show:expr),
                Flip($flip:expr),
                Team($team:expr),
                Flex($flex:expr),
                Color($color:ident),
                Alpha($alpha:ident)
        )+
    ) => {
        #[derive(Debug, Copy, Clone)]
        pub enum GostekPart {
            $($id,)+
        }

        impl GostekPart {
            pub fn id(&self) -> usize { *self as usize }

            pub fn values() -> &'static [GostekPart] {
                static VALUES: &[GostekPart] = &[$(GostekPart::$id,)+];
                VALUES
            }

            pub fn data() -> &'static [GostekPartInfo] {
                static DATA: &[GostekPartInfo] = &[
                    $(
                        GostekPartInfo {
                            name: stringify!($id),
                            sprite: gostek_parts_sprite!($($sprite)+),
                            point: ($p1, $p2),
                            center: ($cx, $cy),
                            flexibility: $flex,
                            flip: $flip,
                            team: $team,
                            color: GostekColor::$color,
                            alpha: GostekAlpha::$alpha,
                            visible: $show,
                        },
                    )+
                ];

                DATA
            }
        }

        impl ::std::convert::From<usize> for GostekPart {
            fn from(id: usize) -> GostekPart {
                match GostekPart::values().get(id as usize) {
                    Some(&v) => v,
                    _ => panic!("Invalid sprite identifier."),
                }
            }
        }

        impl ::std::ops::Add<usize> for GostekPart {
            type Output = GostekPart;
            fn add(self, x: usize) -> GostekPart { GostekPart::from(self.id() + x) }
        }

        impl ::std::ops::Sub<usize> for GostekPart {
            type Output = GostekPart;
            fn sub(self, x: usize) -> GostekPart { GostekPart::from(self.id() - x) }
        }
    }
}
