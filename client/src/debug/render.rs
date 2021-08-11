use super::*;

#[derive(Default)]
pub struct RenderState {
    pub(crate) visible: bool,

    pub render_skeleton: bool,
    pub render_physics: bool,

    // Taken from http://urraka.github.io/soldat-map/#171/Airpirates
    pub disable_background: bool,
    pub disable_polygon: bool,
    pub disable_texture: bool,
    pub render_wireframe: bool,
    pub render_colliders: bool,
    pub disable_scenery: bool,
    pub disable_scenery_back: bool,
    pub disable_scenery_middle: bool,
    pub disable_scenery_front: bool,

    pub render_spawns: bool,
    pub render_spawns_team: [bool; 17],

    pub highlight_polygons: bool,
    pub hlt_poly_normal: bool,
    pub hlt_poly_only_bullets_coll: bool,
    pub hlt_poly_only_players_coll: bool,
    pub hlt_poly_no_coll: bool,
    pub hlt_poly_ice: bool,
    pub hlt_poly_deadly: bool,
    pub hlt_poly_bloody_deadly: bool,
    pub hlt_poly_hurts: bool,
    pub hlt_poly_regenerates: bool,
    pub hlt_poly_lava: bool,
    pub hlt_poly_alpha_bullets: bool,
    pub hlt_poly_alpha_players: bool,
    pub hlt_poly_bravo_bullets: bool,
    pub hlt_poly_bravo_players: bool,
    pub hlt_poly_charlie_bullets: bool,
    pub hlt_poly_charlie_players: bool,
    pub hlt_poly_delta_bullets: bool,
    pub hlt_poly_delta_players: bool,
    pub hlt_poly_bouncy: bool,
    pub hlt_poly_explosive: bool,
    pub hlt_poly_hurt_flaggers: bool,
    pub hlt_poly_flagger_coll: bool,
    pub hlt_poly_non_flagger_coll: bool,
    pub hlt_poly_flag_coll: bool,
    pub hlt_poly_background: bool,
    pub hlt_poly_background_transition: bool,
}

impl IVisit for RenderState {
    fn visit(&mut self, f: &mut dyn FnMut(&mut dyn INode)) {
        f(&mut cvar::Property("visible", &mut self.visible, false));
        f(&mut cvar::Property(
            "render_skeleton",
            &mut self.render_skeleton,
            false,
        ));
        f(&mut cvar::Property(
            "render_physics",
            &mut self.render_physics,
            false,
        ));
        f(&mut cvar::Property(
            "disable_background",
            &mut self.disable_background,
            false,
        ));
        f(&mut cvar::Property(
            "disable_polygon",
            &mut self.disable_polygon,
            false,
        ));
        f(&mut cvar::Property(
            "disable_texture",
            &mut self.disable_texture,
            false,
        ));
        f(&mut cvar::Property(
            "render_wireframe",
            &mut self.render_wireframe,
            false,
        ));
        f(&mut cvar::Property(
            "render_colliders",
            &mut self.render_colliders,
            false,
        ));
        f(&mut cvar::Property(
            "disable_scenery",
            &mut self.disable_scenery,
            false,
        ));
        f(&mut cvar::Property(
            "disable_scenery_back",
            &mut self.disable_scenery_back,
            false,
        ));
        f(&mut cvar::Property(
            "disable_scenery_middle",
            &mut self.disable_scenery_middle,
            false,
        ));
        f(&mut cvar::Property(
            "disable_scenery_front",
            &mut self.disable_scenery_front,
            false,
        ));
    }
}

// impl RenderState {
//     pub fn build_ui(&mut self) {
//         if self.visible {
//             self.visible = widgets::Window::new(hash!(),
//             vec2(980., 10.),
//             vec2(270., 680.))
//             .label("Renderer")
//             .close_button(true)
//             .ui(&mut *root_ui(),
//             |ui| {
//                 toggle_state(ui, None, &mut self.render_skeleton, "Skeleton");
//                 toggle_state(ui, None, &mut self.render_physics, "Physics");

//                 ui.separator();
//                 toggle_state_inv(ui, None, &mut self.disable_background, "Background");
//                 toggle_state_inv(ui, None, &mut self.disable_polygon, "Polygons");
//                 // toggle_state(ui, None, &mut state.disable_texture, "Texture");
//                 toggle_state(ui, None, &mut self.render_wireframe, "Wireframe");
//                 toggle_state(ui, None, &mut self.render_colliders, "Colliders");

//                 ui.tree_node(hash!(), "Scenery", |ui| {
//                     toggle_state_inv(ui, None, &mut self.disable_scenery_back, "Back");
//                     toggle_state_inv(ui, None, &mut self.disable_scenery_middle, "Middle");
//                     toggle_state_inv(ui, None, &mut self.disable_scenery_front, "Front");
//                 });

//                 ui.tree_node(hash!(), "Spawns", |ui| {
//                     toggle_state(ui, None, &mut self.render_spawns_team[0], "General");
//                     toggle_state(ui, None, &mut self.render_spawns_team[1], "Alpha");
//                     toggle_state(ui, None, &mut self.render_spawns_team[2], "Bravo");
//                     toggle_state(ui, None, &mut self.render_spawns_team[3], "Charlie");
//                     toggle_state(ui, None, &mut self.render_spawns_team[4], "Delta");
//                     toggle_state(ui, None, &mut self.render_spawns_team[5], "Alpha Flag");
//                     toggle_state(ui, None, &mut self.render_spawns_team[6], "Bravo Flag");
//                     toggle_state(ui, None, &mut self.render_spawns_team[7], "Grenades");
//                     toggle_state(ui, None, &mut self.render_spawns_team[8], "Medkits");
//                     toggle_state(ui, None, &mut self.render_spawns_team[9], "Clusters");
//                     toggle_state(ui, None, &mut self.render_spawns_team[10], "Vest");
//                     toggle_state(ui, None, &mut self.render_spawns_team[11], "Flamer");
//                     toggle_state(ui, None, &mut self.render_spawns_team[12], "Berserker");
//                     toggle_state(ui, None, &mut self.render_spawns_team[13], "Predator");
//                     toggle_state(ui, None, &mut self.render_spawns_team[14], "Yellow Flag");
//                     toggle_state(ui, None, &mut self.render_spawns_team[15], "Rambo Bow");
//                     toggle_state(ui, None, &mut self.render_spawns_team[16], "Stat Gun");
//                 });
//                 self.render_spawns = self
//                     .render_spawns_team
//                     .iter()
//                     .any(|spawn| *spawn);

//                 #[rustfmt::skip]
//                     ui.tree_node(hash!(), "Highlight polygons", |ui| {
//                         toggle_state(ui, None, &mut self.hlt_poly_normal, "Normal");
//                         toggle_state(ui, None, &mut self.hlt_poly_only_bullets_coll, "Only Bullets Collide");
//                         toggle_state(ui, None, &mut self.hlt_poly_only_players_coll, "Only Players Collide");
//                         toggle_state(ui, None, &mut self.hlt_poly_no_coll, "No Collide");
//                         toggle_state(ui, None, &mut self.hlt_poly_ice, "Ice");
//                         toggle_state(ui, None, &mut self.hlt_poly_deadly, "Deadly");
//                         toggle_state(ui, None, &mut self.hlt_poly_bloody_deadly, "Bloody deadly");
//                         toggle_state(ui, None, &mut self.hlt_poly_hurts, "Hurts");
//                         toggle_state(ui, None, &mut self.hlt_poly_regenerates, "Regenerates");
//                         toggle_state(ui, None, &mut self.hlt_poly_lava, "Lava");
//                         toggle_state(ui, None, &mut self.hlt_poly_alpha_bullets, "Alpha Bullets");
//                         toggle_state(ui, None, &mut self.hlt_poly_alpha_players, "Alpha Players");
//                         toggle_state(ui, None, &mut self.hlt_poly_bravo_bullets, "Bravo Bullets");
//                         toggle_state(ui, None, &mut self.hlt_poly_bravo_players, "Bravo Players");
//                         toggle_state(ui, None, &mut self.hlt_poly_charlie_bullets, "Charlie Bullets");
//                         toggle_state(ui, None, &mut self.hlt_poly_charlie_players, "Charlie Players");
//                         toggle_state(ui, None, &mut self.hlt_poly_delta_bullets, "Delta Bullets");
//                         toggle_state(ui, None, &mut self.hlt_poly_delta_players, "Delta Players");
//                         toggle_state(ui, None, &mut self.hlt_poly_bouncy, "Bouncy");
//                         toggle_state(ui, None, &mut self.hlt_poly_explosive, "Explosive");
//                         toggle_state(ui, None, &mut self.hlt_poly_hurt_flaggers, "Hurt Flaggers");
//                         toggle_state(ui, None, &mut self.hlt_poly_flagger_coll, "Flagger Collides");
//                         toggle_state(ui, None, &mut self.hlt_poly_non_flagger_coll, "Non Flagger Collides");
//                         toggle_state(ui, None, &mut self.hlt_poly_flag_coll, "Flag Collides");
//                         toggle_state(ui, None, &mut self.hlt_poly_background, "Background");
//                         toggle_state(ui, None, &mut self.hlt_poly_background_transition, "Background Transition");
//                                         });
//                 self.highlight_polygons = self.hlt_poly_normal
//                     || self.hlt_poly_only_bullets_coll
//                     || self.hlt_poly_only_players_coll
//                     || self.hlt_poly_no_coll
//                     || self.hlt_poly_ice
//                     || self.hlt_poly_deadly
//                     || self.hlt_poly_bloody_deadly
//                     || self.hlt_poly_hurts
//                     || self.hlt_poly_regenerates
//                     || self.hlt_poly_lava
//                     || self.hlt_poly_alpha_bullets
//                     || self.hlt_poly_alpha_players
//                     || self.hlt_poly_bravo_bullets
//                     || self.hlt_poly_bravo_players
//                     || self.hlt_poly_charlie_bullets
//                     || self.hlt_poly_charlie_players
//                     || self.hlt_poly_delta_bullets
//                     || self.hlt_poly_delta_players
//                     || self.hlt_poly_bouncy
//                     || self.hlt_poly_explosive
//                     || self.hlt_poly_hurt_flaggers
//                     || self.hlt_poly_flagger_coll
//                     || self.hlt_poly_non_flagger_coll
//                     || self.hlt_poly_flag_coll
//                     || self.hlt_poly_background
//                     || self.hlt_poly_background_transition;
//             },
//         );
//         }
//     }
// }
