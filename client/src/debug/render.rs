use super::*;
use megaui_macroquad::megaui::{hash, Ui, Vector2};

#[derive(Default)]
pub struct RenderState {
    pub render_skeleton: bool,
    pub render_position: bool,

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
}

pub fn build_ui(state: &mut DebugState) {
    if state.render_visible {
        let state = &mut state.render;
        draw_window(
            hash!(),
            vec2(980., 10.),
            vec2(270., 680.),
            WindowParams {
                label: "Renderer".to_string(),
                ..Default::default()
            },
            |ui| {
                toggle_state(ui, None, &mut state.render_skeleton, "Skeleton");
                toggle_state(ui, None, &mut state.render_position, "Position");

                ui.separator();
                toggle_state_i(ui, None, &mut state.disable_background, "Background");
                toggle_state_i(ui, None, &mut state.disable_polygon, "Polygons");
                // toggle_state(ui, None, &mut state.disable_texture, "Texture");
                toggle_state(ui, None, &mut state.render_wireframe, "Wireframe");
                toggle_state(ui, None, &mut state.render_colliders, "Colliders");

                ui.tree_node(hash!(), "Scenery", |ui| {
                    toggle_state_i(ui, None, &mut state.disable_scenery_back, "Back");
                    toggle_state_i(ui, None, &mut state.disable_scenery_middle, "Middle");
                    toggle_state_i(ui, None, &mut state.disable_scenery_front, "Front");
                });

                ui.tree_node(hash!(), "Spawns", |ui| {
                    toggle_state(ui, None, &mut state.render_spawns_team[0], "General");
                    toggle_state(ui, None, &mut state.render_spawns_team[1], "Alpha");
                    toggle_state(ui, None, &mut state.render_spawns_team[2], "Bravo");
                    toggle_state(ui, None, &mut state.render_spawns_team[3], "Charlie");
                    toggle_state(ui, None, &mut state.render_spawns_team[4], "Delta");
                    toggle_state(ui, None, &mut state.render_spawns_team[5], "Alpha Flag");
                    toggle_state(ui, None, &mut state.render_spawns_team[6], "Bravo Flag");
                    toggle_state(ui, None, &mut state.render_spawns_team[7], "Grenades");
                    toggle_state(ui, None, &mut state.render_spawns_team[8], "Medkits");
                    toggle_state(ui, None, &mut state.render_spawns_team[9], "Clusters");
                    toggle_state(ui, None, &mut state.render_spawns_team[10], "Vest");
                    toggle_state(ui, None, &mut state.render_spawns_team[11], "Flamer");
                    toggle_state(ui, None, &mut state.render_spawns_team[12], "Berserker");
                    toggle_state(ui, None, &mut state.render_spawns_team[13], "Predator");
                    toggle_state(ui, None, &mut state.render_spawns_team[14], "Yellow Flag");
                    toggle_state(ui, None, &mut state.render_spawns_team[15], "Rambo Bow");
                    toggle_state(ui, None, &mut state.render_spawns_team[16], "Stat Gun");
                });
                state.render_spawns = state
                    .render_spawns_team
                    .iter()
                    .find(|spawn| **spawn)
                    .is_some();

                #[rustfmt::skip]
                    ui.tree_node(hash!(), "Highlight polygons", |ui| {
                        toggle_state(ui, None, &mut state.hlt_poly_normal, "Normal");
                        toggle_state(ui, None, &mut state.hlt_poly_only_bullets_coll, "Only Bullets Collide");
                        toggle_state(ui, None, &mut state.hlt_poly_only_players_coll, "Only Players Collide");
                        toggle_state(ui, None, &mut state.hlt_poly_no_coll, "No Collide");
                        toggle_state(ui, None, &mut state.hlt_poly_ice, "Ice");
                        toggle_state(ui, None, &mut state.hlt_poly_deadly, "Deadly");
                        toggle_state(ui, None, &mut state.hlt_poly_bloody_deadly, "Bloody deadly");
                        toggle_state(ui, None, &mut state.hlt_poly_hurts, "Hurts");
                        toggle_state(ui, None, &mut state.hlt_poly_regenerates, "Regenerates");
                        toggle_state(ui, None, &mut state.hlt_poly_lava, "Lava");
                        toggle_state(ui, None, &mut state.hlt_poly_alpha_bullets, "Alpha Bullets");
                        toggle_state(ui, None, &mut state.hlt_poly_alpha_players, "Alpha Players");
                        toggle_state(ui, None, &mut state.hlt_poly_bravo_bullets, "Bravo Bullets");
                        toggle_state(ui, None, &mut state.hlt_poly_bravo_players, "Bravo Players");
                        toggle_state(ui, None, &mut state.hlt_poly_charlie_bullets, "Charlie Bullets");
                        toggle_state(ui, None, &mut state.hlt_poly_charlie_players, "Charlie Players");
                        toggle_state(ui, None, &mut state.hlt_poly_delta_bullets, "Delta Bullets");
                        toggle_state(ui, None, &mut state.hlt_poly_delta_players, "Delta Players");
                        toggle_state(ui, None, &mut state.hlt_poly_bouncy, "Bouncy");
                        toggle_state(ui, None, &mut state.hlt_poly_explosive, "Explosive");
                        toggle_state(ui, None, &mut state.hlt_poly_hurt_flaggers, "Hurt Flaggers");
                        toggle_state(ui, None, &mut state.hlt_poly_flagger_coll, "Flagger Collides");
                        toggle_state(ui, None, &mut state.hlt_poly_non_flagger_coll, "Non Flagger Collides");
                        toggle_state(ui, None, &mut state.hlt_poly_flag_coll, "Flag Collides");
                    });
                state.highlight_polygons = state.hlt_poly_normal
                    || state.hlt_poly_only_bullets_coll
                    || state.hlt_poly_only_players_coll
                    || state.hlt_poly_no_coll
                    || state.hlt_poly_ice
                    || state.hlt_poly_deadly
                    || state.hlt_poly_bloody_deadly
                    || state.hlt_poly_hurts
                    || state.hlt_poly_regenerates
                    || state.hlt_poly_lava
                    || state.hlt_poly_alpha_bullets
                    || state.hlt_poly_alpha_players
                    || state.hlt_poly_bravo_bullets
                    || state.hlt_poly_bravo_players
                    || state.hlt_poly_charlie_bullets
                    || state.hlt_poly_charlie_players
                    || state.hlt_poly_delta_bullets
                    || state.hlt_poly_delta_players
                    || state.hlt_poly_bouncy
                    || state.hlt_poly_explosive
                    || state.hlt_poly_hurt_flaggers
                    || state.hlt_poly_flagger_coll
                    || state.hlt_poly_non_flagger_coll
                    || state.hlt_poly_flag_coll;
            },
        );
    }
}

fn toggle_state<P: Into<Option<Vector2>>>(ui: &mut Ui, position: P, state: &mut bool, label: &str) {
    if ui.button(position, checkbox_label(*state, label).as_str()) {
        *state = !*state;
    }
}

fn toggle_state_i<P: Into<Option<Vector2>>>(
    ui: &mut Ui,
    position: P,
    state: &mut bool,
    label: &str,
) {
    if ui.button(position, checkbox_label(!*state, label).as_str()) {
        *state = !*state;
    }
}
