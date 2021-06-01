macro_rules! sprites {
    (
        $($enm:ident {
            $($id:ident = $file:tt)+
        })+
    ) => {
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

macro_rules! soldier_parts_sprite {
    ( None ) => {
        SoldierSprite::None
    };
    ( $group:ident::$id:ident ) => {
        SoldierSprite::$group($group::$id)
    };
}

macro_rules! soldier_parts {
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
        pub enum SoldierPart {
            $($id,)+
        }

        impl SoldierPart {
            pub fn id(&self) -> usize { *self as usize }

            pub fn values() -> &'static [SoldierPart] {
                static VALUES: &[SoldierPart] = &[$(SoldierPart::$id,)+];
                VALUES
            }

            pub fn data() -> &'static [SoldierPartInfo] {
                static DATA: &[SoldierPartInfo] = &[
                    $(
                        SoldierPartInfo {
                            name: stringify!($id),
                            sprite: soldier_parts_sprite!($($sprite)+),
                            point: ($p1, $p2),
                            center: ($cx, $cy),
                            flexibility: $flex,
                            flip: $flip,
                            team: $team,
                            color: SoldierColor::$color,
                            alpha: SoldierAlpha::$alpha,
                            visible: $show,
                        },
                    )+
                ];

                DATA
            }
        }
    }
}
