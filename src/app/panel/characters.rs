use egui::{Context, RichText, Color32};
use super::super::{
    TextToolApp, WorldObject, ObjectKind, ObjectLink, LinkTarget, RelationKind,
    StructNode, ObjectViewMode,
};

impl TextToolApp {
    // ── Panel: World Objects ──────────────────────────────────────────────────
    //
    // Left side-panel: object list, object editor form, add form
    // Central panel:   relationship canvas (nodes + connecting lines)

    pub(in crate::app) fn draw_objects_panel(&mut self, ctx: &Context) {
        let mut open_obj: Option<usize> = None;
        let mut remove_obj: Option<usize> = None;
        let mut do_sync = false;
        let mut do_add_link = false;
        let mut remove_link: Option<usize> = None;

        // Collect autocomplete before any mutable borrow (unused for now but needed for future autocomplete)

        egui::SidePanel::left("obj_list")
            .resizable(true)
            .default_width(300.0)
            .min_width(200.0)
            .show(ctx, |ui| {
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.heading("世界对象");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // JSON sync
                        if ui.small_button("⬆").on_hover_text("保存世界对象到 data/world.json").clicked() {
                            do_sync = true;
                        }
                        if ui.small_button("⬇").on_hover_text("从 data/world.json 加载世界对象").clicked() {
                            self.load_world_objects_from_json();
                        }
                        // View mode toggle
                        let is_card = self.obj_view_mode == ObjectViewMode::Card;
                        if ui.selectable_label(is_card, "🃏").on_hover_text("卡片视图").clicked() {
                            self.obj_view_mode = ObjectViewMode::Card;
                        }
                        if ui.selectable_label(!is_card, "☰").on_hover_text("列表视图").clicked() {
                            self.obj_view_mode = ObjectViewMode::List;
                        }
                    });
                });
                // Kind filter chips
                ui.horizontal_wrapped(|ui| {
                    let all_sel = self.obj_kind_filter.is_none();
                    if ui.selectable_label(all_sel, "全部").clicked() {
                        self.obj_kind_filter = None;
                    }
                    for k in ObjectKind::all() {
                        let sel = self.obj_kind_filter.as_ref() == Some(k);
                        if ui.selectable_label(sel,
                            format!("{} {}", k.icon(), k.label())).clicked()
                        {
                            self.obj_kind_filter = if sel { None } else { Some(k.clone()) };
                        }
                    }
                });
                ui.separator();

                // ── Object list (top portion) ──────────────────────────────────
                let list_height = 160.0_f32;
                egui::ScrollArea::vertical()
                    .id_salt("obj_list_scroll")
                    .max_height(list_height)
                    .show(ui, |ui| {
                        if self.obj_view_mode == ObjectViewMode::List {
                            let mut pending_move: Option<(usize, usize)> = None;
                            for i in 0..self.world_objects.len() {
                                let obj = &self.world_objects[i];
                                if let Some(ref filter) = self.obj_kind_filter {
                                    if &obj.kind != filter { continue; }
                                }
                                let selected = self.selected_obj_idx == Some(i);
                                let label = format!("{} {}", obj.icon(), obj.name);
                                let item_id = egui::Id::new(("wo_drag", i));
                                let ir = ui.dnd_drag_source(item_id, i, |ui| {
                                    ui.selectable_label(selected, &label)
                                });
                                if let Some(payload) = ir.response.dnd_release_payload::<usize>() {
                                    let from = *payload;
                                    if from != i { pending_move = Some((from, i)); }
                                }
                                ir.response.context_menu(|ui| {
                                    if ui.button("删除").clicked() {
                                        remove_obj = Some(i);
                                        ui.close_menu();
                                    }
                                });
                                if ir.inner.clicked() { open_obj = Some(i); }
                            }
                            if let Some((from, to)) = pending_move {
                                if from < self.world_objects.len() && to < self.world_objects.len() {
                                    let item = self.world_objects.remove(from);
                                    self.world_objects.insert(to, item);
                                    if let Some(sel) = self.selected_obj_idx {
                                        if sel == from {
                                            self.selected_obj_idx = Some(to);
                                        } else if from < to && sel > from && sel <= to {
                                            self.selected_obj_idx = Some(sel - 1);
                                        } else if from > to && sel >= to && sel < from {
                                            self.selected_obj_idx = Some(sel + 1);
                                        }
                                    }
                                }
                            }
                        } else {
                            for (i, obj) in self.world_objects.iter().enumerate() {
                                if let Some(ref filter) = self.obj_kind_filter {
                                    if &obj.kind != filter { continue; }
                                }
                                let selected = self.selected_obj_idx == Some(i);
                                let bg = if selected { Color32::from_rgb(0, 80, 140) } else { Color32::from_gray(38) };
                                let card_resp = egui::Frame::none()
                                    .fill(bg).rounding(6.0)
                                    .inner_margin(egui::Margin::symmetric(8.0, 4.0))
                                    .show(ui, |ui| {
                                        ui.set_min_width(ui.available_width());
                                        ui.horizontal(|ui| {
                                            ui.label(RichText::new(obj.icon()).size(18.0));
                                            ui.label(RichText::new(&obj.name).strong());
                                            ui.label(RichText::new(obj.kind.label()).small().color(Color32::from_gray(160)));
                                        });
                                    }).response.interact(egui::Sense::click());
                                card_resp.context_menu(|ui| {
                                    if ui.button("删除").clicked() {
                                        remove_obj = Some(i);
                                        ui.close_menu();
                                    }
                                });
                                if card_resp.clicked() { open_obj = Some(i); }
                                ui.add_space(2.0);
                            }
                        }
                    });

                // ── Quick-add form ─────────────────────────────────────────────
                ui.separator();
                ui.horizontal(|ui| {
                    ui.add(egui::TextEdit::singleline(&mut self.new_obj_name)
                        .hint_text("新对象名称").desired_width(100.0));
                    egui::ComboBox::from_id_salt("new_obj_kind")
                        .selected_text(format!("{} {}", self.new_obj_kind.icon(), self.new_obj_kind.label()))
                        .width(80.0)
                        .show_ui(ui, |ui| {
                            for k in ObjectKind::all() {
                                let label = format!("{} {}", k.icon(), k.label());
                                ui.selectable_value(&mut self.new_obj_kind, k.clone(), label);
                            }
                        });
                    if ui.button("➕").on_hover_text("添加新对象").clicked() {
                        let name = self.new_obj_name.trim().to_owned();
                        if !name.is_empty() {
                            let idx = self.world_objects.len();
                            self.world_objects.push(WorldObject::new(&name, self.new_obj_kind.clone()));
                            self.selected_obj_idx = Some(idx);
                            self.new_obj_name.clear();
                        }
                    }
                });
                ui.separator();

                // ── Selected-object detail editor ──────────────────────────────
                if let Some(idx) = self.selected_obj_idx {
                    if idx < self.world_objects.len() {
                        egui::ScrollArea::vertical().id_salt("obj_detail_scroll").show(ui, |ui| {
                            let obj = &mut self.world_objects[idx];

                            ui.horizontal(|ui| {
                                ui.label(RichText::new(obj.icon()).size(18.0));
                                ui.text_edit_singleline(&mut obj.name);
                            });

                            ui.add_space(2.0);
                            ui.label("描述 / 核心特质:");
                            ui.add(egui::TextEdit::multiline(&mut obj.description)
                                .desired_rows(2).desired_width(f32::INFINITY));

                            ui.add_space(2.0);
                            ui.label("背景故事:");
                            ui.add(egui::TextEdit::multiline(&mut obj.background)
                                .desired_rows(3).desired_width(f32::INFINITY));

                            ui.add_space(4.0);
                            ui.separator();
                            ui.label(RichText::new("关联").strong());

                            if obj.links.is_empty() {
                                ui.label(RichText::new("（暂无关联）").color(Color32::GRAY).small());
                            } else {
                                for (li, link) in obj.links.iter().enumerate() {
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new(link.target.type_label()).small()
                                            .color(Color32::from_rgb(120, 180, 240)));
                                        ui.label(RichText::new(link.target.display_name()).small());
                                        ui.label(RichText::new(link.kind.label()).small());
                                        if ui.small_button("🗑").clicked() {
                                            remove_link = Some(li);
                                        }
                                    });
                                }
                            }

                            if let Some(li) = remove_link { obj.links.remove(li); }

                            ui.add_space(4.0);
                            ui.label(RichText::new("添加关联:").small());
                            ui.horizontal(|ui| {
                                if ui.selectable_label(!self.new_link_is_node, "对象").clicked() { self.new_link_is_node = false; }
                                if ui.selectable_label(self.new_link_is_node, "章节").clicked() { self.new_link_is_node = true; }
                            });
                            ui.horizontal(|ui| {
                                let hint = if self.new_link_is_node { "节点标题" } else { "对象名称" };
                                ui.add(egui::TextEdit::singleline(&mut self.new_link_name)
                                    .hint_text(hint).desired_width(90.0));
                                egui::ComboBox::from_id_salt("new_link_rel")
                                    .selected_text(self.new_link_rel_kind.label())
                                    .width(70.0)
                                    .show_ui(ui, |ui| {
                                        for k in RelationKind::all() {
                                            ui.selectable_value(&mut self.new_link_rel_kind, k.clone(), k.label());
                                        }
                                    });
                                if ui.button("➕").clicked() {
                                    let name = self.new_link_name.trim().to_owned();
                                    if !name.is_empty() { do_add_link = true; }
                                }
                            });
                            ui.horizontal(|ui| {
                                ui.label("备注:");
                                ui.add(egui::TextEdit::singleline(&mut self.new_link_note)
                                    .desired_width(f32::INFINITY));
                            });
                        });
                    }
                } else {
                    ui.label(RichText::new("← 点击对象以编辑").color(Color32::GRAY));
                }
            });

        // Apply deferred mutations
        if let Some(i) = open_obj { self.selected_obj_idx = Some(i); }
        if let Some(i) = remove_obj {
            self.world_objects.remove(i);
            match self.selected_obj_idx {
                Some(s) if s == i => self.selected_obj_idx = None,
                Some(s) if s > i  => self.selected_obj_idx = Some(s - 1),
                _ => {}
            }
        }
        if do_add_link {
            let name = self.new_link_name.trim().to_owned();
            let target = if self.new_link_is_node {
                LinkTarget::Node(name)
            } else {
                LinkTarget::Object(name)
            };
            if let Some(idx) = self.selected_obj_idx {
                if let Some(obj) = self.world_objects.get_mut(idx) {
                    obj.links.push(ObjectLink {
                        target,
                        kind: self.new_link_rel_kind.clone(),
                        note: self.new_link_note.trim().to_owned(),
                    });
                }
            }
            self.new_link_name.clear();
            self.new_link_note.clear();
        }
        if do_sync { self.sync_world_objects_to_json(); }

        // ── Central: relationship canvas ───────────────────────────────────────
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("关系图谱");
            ui.separator();

            let sel_idx = self.selected_obj_idx;
            if sel_idx.is_none() || self.world_objects.is_empty() {
                ui.centered_and_justified(|ui| {
                    ui.label(
                        RichText::new("← 在左侧选中一个对象\n可在此查看其关联关系图谱")
                            .color(Color32::GRAY),
                    );
                });
                return;
            }
            let idx = sel_idx.unwrap();
            if idx >= self.world_objects.len() { return; }

            let obj = &self.world_objects[idx];
            let obj_name = obj.name.clone();
            let links: Vec<(String, String, String)> = obj.links.iter()
                .map(|l| (
                    l.target.display_name().to_owned(),
                    l.kind.label().to_owned(),
                    l.target.type_label().to_owned(),
                ))
                .collect();

            // Reverse-lookup: nodes that link back to this object
            let reverse = Self::collect_nodes_linking_object(&self.struct_roots, &obj_name);

            // Allocate the canvas area
            let available = ui.available_size();
            let (resp, painter) = ui.allocate_painter(available, egui::Sense::hover());
            let rect = resp.rect;
            let center = rect.center();

            // Draw center node
            let center_radius = 42.0_f32;
            let node_color = Color32::from_rgb(0, 100, 180);
            painter.circle_filled(center, center_radius, node_color);
            painter.circle_stroke(center, center_radius, egui::Stroke::new(2.0, Color32::from_rgb(60, 160, 255)));
            painter.text(
                center,
                egui::Align2::CENTER_CENTER,
                format!("{}\n({})", obj_name, obj.kind.label()),
                egui::FontId::proportional(12.0),
                Color32::WHITE,
            );

            // Draw linked nodes in a ring
            let total = links.len() + reverse.len();
            if total > 0 {
                let radius = (available.x.min(available.y) * 0.38).max(120.0);
                let mut i = 0usize;

                // Forward links (direct)
                for (target_name, rel_label, type_label) in &links {
                    let angle = i as f32 * std::f32::consts::TAU / total as f32
                        - std::f32::consts::FRAC_PI_2;
                    let node_pos = center + egui::vec2(angle.cos() * radius, angle.sin() * radius);

                    // Line from center to node
                    painter.line_segment(
                        [center + egui::vec2(angle.cos() * center_radius, angle.sin() * center_radius),
                         node_pos - egui::vec2(angle.cos() * 24.0, angle.sin() * 24.0)],
                        egui::Stroke::new(1.5, Color32::from_gray(130)),
                    );
                    // Arrowhead
                    let tip = node_pos - egui::vec2(angle.cos() * 24.0, angle.sin() * 24.0);
                    let perp = egui::vec2(-angle.sin(), angle.cos()) * 5.0;
                    let back = egui::vec2(-angle.cos(), -angle.sin()) * 10.0;
                    painter.add(egui::epaint::Shape::convex_polygon(
                        vec![tip, tip + back + perp, tip + back - perp],
                        Color32::from_gray(150),
                        egui::Stroke::NONE,
                    ));
                    // Relation label on the line midpoint
                    let mid = egui::pos2((center.x + node_pos.x) * 0.5, (center.y + node_pos.y) * 0.5);
                    painter.text(
                        mid,
                        egui::Align2::CENTER_CENTER,
                        rel_label,
                        egui::FontId::proportional(9.0),
                        Color32::from_rgb(180, 200, 150),
                    );

                    // Satellite node (color varies by type)
                    let sat_color = if type_label == "章节" {
                        Color32::from_rgb(80, 130, 60)
                    } else {
                        Color32::from_rgb(80, 80, 150)
                    };
                    painter.circle_filled(node_pos, 24.0, sat_color);
                    painter.circle_stroke(node_pos, 24.0, egui::Stroke::new(1.5, Color32::from_gray(160)));
                    painter.text(
                        node_pos,
                        egui::Align2::CENTER_CENTER,
                        target_name,
                        egui::FontId::proportional(10.0),
                        Color32::WHITE,
                    );
                    i += 1;
                }

                // Reverse links (nodes that reference this object)
                for rev_title in &reverse {
                    let angle = i as f32 * std::f32::consts::TAU / total as f32
                        - std::f32::consts::FRAC_PI_2;
                    let node_pos = center + egui::vec2(angle.cos() * radius, angle.sin() * radius);

                    painter.line_segment(
                        [node_pos + egui::vec2(angle.cos() * 24.0, angle.sin() * 24.0),
                         center - egui::vec2(angle.cos() * center_radius, angle.sin() * center_radius)],
                        egui::Stroke::new(1.2, Color32::from_rgb(120, 180, 120)),
                    );
                    let mid = egui::pos2((center.x + node_pos.x) * 0.5, (center.y + node_pos.y) * 0.5);
                    painter.text(
                        mid,
                        egui::Align2::CENTER_CENTER,
                        "出现",
                        egui::FontId::proportional(9.0),
                        Color32::from_rgb(120, 200, 120),
                    );
                    painter.circle_filled(node_pos, 22.0, Color32::from_rgb(50, 100, 50));
                    painter.circle_stroke(node_pos, 22.0, egui::Stroke::new(1.5, Color32::from_rgb(100, 200, 100)));
                    painter.text(
                        node_pos,
                        egui::Align2::CENTER_CENTER,
                        rev_title,
                        egui::FontId::proportional(10.0),
                        Color32::WHITE,
                    );
                    i += 1;
                }
            }

            // Legend (top-left overlay)
            let legend_pos = rect.min + egui::vec2(8.0, 8.0);
            let legend_bg = egui::Rect::from_min_size(legend_pos, egui::vec2(150.0, 60.0));
            painter.rect_filled(legend_bg, 4.0, Color32::from_black_alpha(140));
            painter.text(legend_pos + egui::vec2(6.0, 4.0), egui::Align2::LEFT_TOP,
                "图例", egui::FontId::proportional(10.0), Color32::from_gray(180));
            let items: &[(&str, Color32)] = &[
                ("■ 关联对象", Color32::from_rgb(80, 80, 150)),
                ("■ 关联章节", Color32::from_rgb(80, 130, 60)),
                ("■ 出现于章节", Color32::from_rgb(50, 100, 50)),
            ];
            for (i, (label, color)) in items.iter().enumerate() {
                painter.text(
                    legend_pos + egui::vec2(6.0, 18.0 + i as f32 * 14.0),
                    egui::Align2::LEFT_TOP,
                    *label,
                    egui::FontId::proportional(9.0),
                    *color,
                );
            }
        });
    }

    /// Collect titles of all `StructNode`s that list `obj_name` in their `linked_objects`.
    fn collect_nodes_linking_object(roots: &[StructNode], obj_name: &str) -> Vec<String> {
        let mut out = Vec::new();
        fn walk(nodes: &[StructNode], name: &str, out: &mut Vec<String>) {
            for n in nodes {
                if n.linked_objects.iter().any(|o| o == name) {
                    out.push(n.title.clone());
                }
                walk(&n.children, name, out);
            }
        }
        walk(roots, obj_name, &mut out);
        out
    }
}
