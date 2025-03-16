use super::*;
use std::collections::HashMap;

#[derive(Default)]
pub struct EntitiesState {
    pub(crate) visible: bool,
}

impl IVisit for EntitiesState {
    fn visit(&mut self, f: &mut dyn FnMut(&mut dyn INode)) {
        f(&mut cvar::Property("visible", &mut self.visible, false));
    }
}

impl EntitiesState {
    pub fn build_ui(&mut self, egui_ctx: &egui::Context, world: &mut World) {
        if self.visible {
            let mut visible = self.visible;

            let mut entities = world.iter().collect::<Vec<_>>();
            entities.sort_by_key(|ent| ent.entity());

            let mut archetypes = HashMap::new();
            for archetype in world.archetypes() {
                for id in archetype.ids() {
                    archetypes.insert(id, format!("{:p}", archetype as *const _));
                }
            }

            egui::Window::new(format!("ECS Entities ({})", entities.len()).as_str())
                .open(&mut visible)
                .resizable(true)
                .scroll(true)
                .show(egui_ctx, |ui| {
                    for entity_ref in entities.iter() {
                        let entity = entity_ref.entity();
                        let unknown = "???".to_string();
                        ui.collapsing(
                            format!(
                                "{:?}  @{}",
                                entity,
                                archetypes.get(&entity.id()).unwrap_or(&unknown)
                            )
                            .as_str(),
                            |ui| {
                                for type_id in entity_ref.component_types() {
                                    ui.label(format!("{:?}", type_id).as_str());
                                }
                            },
                        );
                        // if let Some(location) = world.get_entity_location(*entity) {
                        //     let archetype = &world.archetypes[location.archetype as usize];
                        //     ui.tree_node(
                        //         hash!(entity.id()),
                        //         format!("{:?}  @{}", entity, location.archetype).as_str(),
                        //         |ui| {
                        //             for type_info in archetype.types() {
                        //                 let mut serialized = false;

                        //                 if let Some(registration) = registry.get(type_info.id()) {
                        //                     if let Some(reflect_component) =
                        //                         registration.data::<ReflectComponent>()
                        //                     {
                        //                         let serializer = unsafe {
                        //                             // SAFE: the index comes directly from a currently live component
                        //                             let component = reflect_component
                        //                                 .reflect_component(&archetype, location.index);
                        //                             ReflectSerializer::new(component, &registry)
                        //                         };

                        //                         let json = serde_json::to_string(&serializer).unwrap();
                        //                         let v: serde_json::Value =
                        //                             serde_json::from_str(json.as_ref()).unwrap();

                        //                         if let Some(map) = v.as_object() {
                        //                             if let Some(s) = map["type"].as_str() {
                        //                                 serialized = true;
                        //                                 ui.tree_node(
                        //                                     hash!(entity.id(), s),
                        //                                     trim_type_name(s),
                        //                                     |ui| {
                        //                                         if let Some(obj) = map
                        //                                             .get("struct")
                        //                                             .map(|o| o.as_object().unwrap())
                        //                                         {
                        //                                             for (name, value) in obj.iter() {
                        //                                                 ui.label(
                        //                                                     None,
                        //                                                     format!(
                        //                                                         " {}({}) = {}",
                        //                                                         name,
                        //                                                         strip_type_name(
                        //                                                             value
                        //                                                                 .get("type")
                        //                                                                 .map(|t| t
                        //                                                                     .as_str()
                        //                                                                     .unwrap())
                        //                                                         ),
                        //                                                         value["value"]
                        //                                                     )
                        //                                                     .as_str(),
                        //                                                 );
                        //                                             }
                        //                                         } else if let Some(arr) = map
                        //                                             .get("tuple_struct")
                        //                                             .map(|o| o.as_array().unwrap())
                        //                                         {
                        //                                             for (index, value) in
                        //                                                 arr.iter().enumerate()
                        //                                             {
                        //                                                 ui.label(
                        //                                                     None,
                        //                                                     format!(
                        //                                                         " {} ({}) = {}",
                        //                                                         index,
                        //                                                         strip_type_name(
                        //                                                             value
                        //                                                                 .get("type")
                        //                                                                 .map(|t| t
                        //                                                                     .as_str()
                        //                                                                     .unwrap())
                        //                                                         ),
                        //                                                         value["value"]
                        //                                                     )
                        //                                                     .as_str(),
                        //                                                 );
                        //                                             }
                        //                                         } else {
                        //                                             ui.label(None, json.as_str());
                        //                                         }
                        //                                     },
                        //                                 );
                        //                             }
                        //                         }
                        //                     }
                        //                 }

                        //                 if !serialized {
                        //                     ui.label(
                        //                         None,
                        //                         format!(" {}", trim_type_name(type_info.type_name()))
                        //                             .as_str(),
                        //                     );
                        //                 }
                        //             }
                        //         },
                        //     );
                        // }
                    }
                });
            self.visible = visible;
        }
    }
}

fn trim_type_name(name: &str) -> &str {
    name.strip_prefix("bv_soldank::").unwrap_or(name)
}

fn strip_type_name(name: Option<&str>) -> &str {
    match name {
        Some(name) => name.split("::").last().unwrap(),
        None => "???",
    }
}
