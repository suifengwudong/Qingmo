use egui::{RichText, Color32};
use super::super::{TextToolApp, LlmTask, PromptTemplate, LlmHistoryEntry, unix_secs_to_iso_date};

impl TextToolApp {
    // ── Panel: LLM Assistance ─────────────────────────────────────────────────

    pub(in crate::app) fn draw_llm_panel(&mut self, ctx: &egui::Context) {
        // Poll for completed background task each frame
        if let Some(task) = &self.llm_task {
            match task.receiver.try_recv() {
                Ok(Ok(text)) => {
                    self.llm_output = text.clone();
                    self.status = "LLM 补全完成".to_owned();
                    self.llm_task = None;
                    // ── Persist to history ────────────────────────────────────
                    if let Some(hist_path) = self.llm_history_path.clone() {
                        let ts = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs();
                        let session_key = self.llm_session_key();
                        let entry_id = self.llm_history.alloc_id();
                        let entry = LlmHistoryEntry {
                            id: entry_id,
                            timestamp: ts,
                            session_key,
                            prompt: self.llm_prompt.clone(),
                            response: text,
                            model: self.llm_config.model_path.clone(),
                        };
                        let _ = self.llm_history.append(entry, &hist_path);
                    }
                    ctx.request_repaint();
                }
                Ok(Err(e)) => {
                    self.llm_output = format!("【错误】{e}");
                    self.status = format!("LLM 调用失败: {e}");
                    self.llm_task = None;
                    ctx.request_repaint();
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    ctx.request_repaint();
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.llm_output = "【错误】后台线程意外断开".to_owned();
                    self.llm_task = None;
                }
            }
        }

        let is_running = self.llm_task.is_some();

        // Collect names before mutable borrows below.
        let char_names: Vec<String> = self.world_objects.iter()
            .map(|o| o.name.clone())
            .collect();

        egui::SidePanel::left("llm_config")
            .resizable(true)
            .default_width(260.0)
            .min_width(180.0)
            .show(ctx, |ui| {
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.heading("LLM 配置");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            RichText::new(self.current_backend_name())
                                .small()
                                .color(Color32::from_rgb(100, 200, 120)),
                        );
                    });
                });
                ui.separator();

                // ── Backend selector ───────────────────────────────────────────
                ui.label("接口类型:");
                ui.horizontal_wrapped(|ui| {
                    if ui.selectable_label(self.llm_backend_idx == 0, "🧪 模拟模型").clicked() {
                        self.llm_backend_idx = 0;
                    }
                    if ui.selectable_label(self.llm_backend_idx == 1, "🌐 HTTP API").clicked() {
                        self.llm_backend_idx = 1;
                    }
                    if ui.selectable_label(self.llm_backend_idx == 2, "🖥 本地服务器").clicked() {
                        self.llm_backend_idx = 2;
                    }
                    if ui.selectable_label(self.llm_backend_idx == 3, "⚡ Agent").clicked() {
                        self.llm_backend_idx = 3;
                    }
                });
                ui.add_space(4.0);
                ui.separator();

                match self.llm_backend_idx {
                    1 => {
                        // ── HTTP API (Ollama / OpenAI) ─────────────────────────
                        ui.checkbox(&mut self.llm_config.use_local, "本地模型 (Ollama)");
                        ui.add_space(4.0);
                        if self.llm_config.use_local {
                            ui.label("模型名称:");
                            ui.text_edit_singleline(&mut self.llm_config.model_path)
                                .on_hover_text("Ollama 模型名称，如 llama2、phi3 等");
                            ui.add_space(4.0);
                            ui.label("API 地址:");
                            ui.text_edit_singleline(&mut self.llm_config.api_url)
                                .on_hover_text("默认: http://localhost:11434/api/generate");
                        } else {
                            ui.label("API 地址 (OpenAI 兼容):");
                            ui.text_edit_singleline(&mut self.llm_config.api_url)
                                .on_hover_text("如 https://api.openai.com/v1/chat/completions");
                            ui.add_space(4.0);
                            ui.label("模型名称:");
                            ui.text_edit_singleline(&mut self.llm_config.model_path)
                                .on_hover_text("如 gpt-4o、gpt-3.5-turbo 等");
                        }
                        ui.add_space(6.0);
                        ui.label("系统提示词 (可选):");
                        ui.add(egui::TextEdit::multiline(&mut self.llm_config.system_prompt)
                            .desired_rows(3)
                            .desired_width(f32::INFINITY)
                            .hint_text("例如：你是一个专业的小说编辑，请用中文回复。"));
                    }
                    2 => {
                        // ── Local llama.cpp server ─────────────────────────────
                        ui.label(
                            RichText::new("启动 llama.cpp 服务器:\n./server -m model.gguf \\\n  -c 2048 --port 8080")
                                .color(Color32::from_gray(150))
                                .small()
                                .monospace(),
                        );
                        ui.add_space(4.0);
                        ui.label("服务器地址:");
                        ui.text_edit_singleline(&mut self.llm_config.api_url)
                            .on_hover_text("默认: http://127.0.0.1:8080");
                        ui.add_space(6.0);
                        ui.label("系统提示词 (可选):");
                        ui.add(egui::TextEdit::multiline(&mut self.llm_config.system_prompt)
                            .desired_rows(3)
                            .desired_width(f32::INFINITY)
                            .hint_text("例如：你是一个专业的小说编辑，请用中文回复。"));
                    }
                    3 => {
                        // ── Agent (tool-calling loop) ──────────────────────────
                        ui.label(
                            RichText::new("需要支持工具调用的 OpenAI 兼容 API\n（如 gpt-4o、deepseek-chat）")
                                .color(Color32::from_rgb(200, 180, 80))
                                .small(),
                        );
                        ui.add_space(4.0);
                        ui.label("API 地址:");
                        ui.text_edit_singleline(&mut self.llm_config.api_url)
                            .on_hover_text("如 https://api.openai.com/v1/chat/completions");
                        ui.add_space(4.0);
                        ui.label("模型名称:");
                        ui.text_edit_singleline(&mut self.llm_config.model_path)
                            .on_hover_text("如 gpt-4o、deepseek-chat 等");
                        ui.add_space(6.0);
                        ui.label("系统提示词 (可选):");
                        ui.add(egui::TextEdit::multiline(&mut self.llm_config.system_prompt)
                            .desired_rows(2)
                            .desired_width(f32::INFINITY)
                            .hint_text("留空时自动注入项目数据作为上下文"));
                        ui.add_space(6.0);
                        ui.separator();
                        ui.label(RichText::new("当前可用技能 (最多 5 轮调用):").small()
                            .color(Color32::from_gray(160)));
                        // Read-only skills
                        for (name, desc) in &[
                            ("list_characters",    "列出所有世界对象"),
                            ("get_character_info", "获取人物/对象详情"),
                            ("get_chapter_outline","获取章节结构大纲"),
                            ("search_foreshadows", "搜索伏笔列表"),
                            ("get_milestone_status","获取里程碑状态"),
                            ("list_project_files", "列出项目文件"),
                            ("get_file_content",   "读取文件内容"),
                            ("get_text_templates", "获取写作模板"),
                        ] {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("🔍").small()
                                    .color(Color32::from_rgb(100, 170, 230)));
                                ui.label(RichText::new(*name).small().monospace())
                                    .on_hover_text(*desc);
                            });
                        }
                        // Write/mutation skills
                        ui.add_space(2.0);
                        ui.label(RichText::new("写入技能:").small().color(Color32::from_gray(140)));
                        for (name, desc) in &[
                            ("add_world_object",    "添加世界对象"),
                            ("update_world_object", "更新世界对象"),
                            ("delete_world_object", "删除世界对象"),
                            ("add_chapter_node",    "添加章节节点"),
                            ("add_foreshadow",      "添加伏笔"),
                            ("resolve_foreshadow",  "标记伏笔已解决"),
                            ("write_file_content",  "写入项目文件"),
                        ] {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("✏").small()
                                    .color(Color32::from_rgb(200, 140, 60)));
                                ui.label(RichText::new(*name).small().monospace())
                                    .on_hover_text(*desc);
                            });
                        }
                    }
                    _ => {
                        // ── Mock ───────────────────────────────────────────────
                        ui.label(
                            RichText::new("使用内置模拟模型，\n无需配置。")
                                .color(Color32::from_gray(150))
                                .small(),
                        );
                    }
                }

                ui.add_space(8.0);
                ui.label(format!("温度 (Temperature): {:.2}", self.llm_config.temperature));
                ui.add(egui::Slider::new(&mut self.llm_config.temperature, 0.0..=2.0)
                    .step_by(0.05));

                ui.add_space(4.0);
                ui.label(format!("最大 Token: {}", self.llm_config.max_tokens));
                ui.add(egui::Slider::new(&mut self.llm_config.max_tokens, 64..=2048)
                    .step_by(64.0));

                ui.add_space(8.0);
                ui.separator();
                ui.label(
                    RichText::new("支持后端:\n🧪 模拟模型 (无需网络)\n🌐 Ollama / OpenAI API\n🖥 llama.cpp HTTP 服务器")
                        .color(Color32::from_gray(140))
                        .small(),
                );
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            // ── Tab bar ────────────────────────────────────────────────────────
            ui.horizontal(|ui| {
                if ui.selectable_label(self.llm_panel_tab == 0, "📝 任务").clicked() {
                    self.llm_panel_tab = 0;
                }
                if ui.selectable_label(self.llm_panel_tab == 1, "📋 历史").clicked() {
                    self.llm_panel_tab = 1;
                }
            });
            ui.separator();

            match self.llm_panel_tab {
                1 => self.draw_llm_history_tab(ui),
                _ => self.draw_llm_task_tab(ui, is_running, &char_names),
            }
        });
    }

    // ── Task tab ──────────────────────────────────────────────────────────────

    fn draw_llm_task_tab(&mut self, ui: &mut egui::Ui, is_running: bool, char_names: &[String]) {
        ui.heading("LLM 辅助写作");
        ui.separator();

        // ── Prompt templates ───────────────────────────────────────────────────
        // Snapshot context once; used only when a template button is clicked.
        let char_ctx = self.build_character_context();
        ui.label(RichText::new("快速模板:").small().color(Color32::from_gray(160)));
        ui.horizontal_wrapped(|ui| {
            for tmpl in PromptTemplate::all() {
                if ui.small_button(tmpl.label()).clicked() {
                    let current = self.llm_prompt.clone();
                    self.llm_prompt = tmpl.fill(&char_ctx, &current);
                    self.status = format!("已应用模板: {}", tmpl.label());
                }
            }
        });

        ui.add_space(4.0);
        ui.separator();

        // ── Structured context injection ───────────────────────────────────────
        ui.label(RichText::new("注入结构化上下文 (追加到提示词末尾):").small()
            .color(Color32::from_gray(160)));
        ui.horizontal_wrapped(|ui| {
            if ui.button("👤 注入人物信息").clicked() {
                let ctx_text = self.build_character_context();
                if ctx_text.is_empty() {
                    self.status = "世界对象面板中暂无人物，请先添加".to_owned();
                } else {
                    self.llm_prompt.push_str("\n\n");
                    self.llm_prompt.push_str(&ctx_text);
                    self.status = "已注入人物/世界对象信息".to_owned();
                }
            }
            if ui.button("📖 注入章节结构").clicked() {
                let ctx_text = self.build_structure_context();
                if ctx_text.is_empty() {
                    self.status = "章节结构面板中暂无内容，请先添加".to_owned();
                } else {
                    self.llm_prompt.push_str("\n\n");
                    self.llm_prompt.push_str(&ctx_text);
                    self.status = "已注入章节结构信息".to_owned();
                }
            }
        });

        // ── Dialogue style optimisation ────────────────────────────────────────
        ui.add_space(4.0);
        ui.separator();
        ui.label(RichText::new("人设对话风格优化:").small().color(Color32::from_gray(160)));
        ui.horizontal(|ui| {
            ui.label("选择人物:");
            egui::ComboBox::from_id_salt("dialogue_char_picker")
                .selected_text(if self.llm_dialogue_char.is_empty() {
                    "（未选择）".to_owned()
                } else {
                    self.llm_dialogue_char.clone()
                })
                .width(130.0)
                .show_ui(ui, |ui| {
                    for name in char_names {
                        ui.selectable_value(
                            &mut self.llm_dialogue_char,
                            name.clone(),
                            name,
                        );
                    }
                });

            let can_optimize = !self.llm_dialogue_char.is_empty()
                && !self.llm_prompt.trim().is_empty();
            ui.add_enabled_ui(can_optimize, |ui| {
                if ui.button("✨ 优化对话风格").clicked() {
                    let char_name = self.llm_dialogue_char.clone();
                    let dialogue_text = self.llm_prompt.clone();
                    if let Some(prompt) =
                        self.build_dialogue_optimization_prompt(&char_name, &dialogue_text)
                    {
                        let backend = self.make_llm_backend();
                        let config  = self.llm_config.clone();
                        self.llm_task = Some(LlmTask::spawn(backend, config, prompt));
                        self.status = format!("正在优化「{}」的对话风格…", char_name);
                    } else {
                        self.status = format!(
                            "未找到人物「{}」，请先在世界对象面板中添加",
                            char_name
                        );
                    }
                }
            });
        });
        if char_names.is_empty() {
            ui.label(
                RichText::new("  ← 请先在「世界对象」面板中添加人物")
                    .small()
                    .color(Color32::from_gray(120)),
            );
        }

        ui.add_space(6.0);
        ui.separator();

        // ── Prompt editor ──────────────────────────────────────────────────────
        ui.label("提示词 / 上下文:");
        egui::ScrollArea::vertical()
            .id_salt("llm_prompt_scroll")
            .max_height(180.0)
            .show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut self.llm_prompt)
                        .desired_width(f32::INFINITY)
                        .desired_rows(7)
                        .hint_text("输入提示词，例如：\n续写以下场景：\n或 优化以下对话：\n\n也可用上方快速模板或注入按钮自动填充。")
                );
            });

        ui.add_space(4.0);
        ui.horizontal(|ui| {
            if is_running {
                ui.add(egui::Spinner::new());
                ui.label(RichText::new("正在调用 LLM…").color(Color32::from_rgb(200, 200, 80)));
                if ui.button("⏹ 取消").clicked() {
                    self.llm_task = None;
                    self.status = "已取消 LLM 调用".to_owned();
                }
            } else {
                if ui.button("▶ 调用 LLM 补全").clicked() {
                    let backend = self.make_llm_backend();
                    let config  = self.llm_config.clone();
                    let prompt  = self.llm_prompt.clone();
                    self.llm_task = Some(LlmTask::spawn(backend, config, prompt));
                    self.status = "LLM 调用已提交，后台处理中…".to_owned();
                }
                if ui.button("插入到左侧编辑区").clicked()
                    && !self.llm_output.is_empty() {
                        if let Some(lf) = &mut self.left_file {
                            lf.content.push_str("\n\n");
                            lf.content.push_str(&self.llm_output);
                            lf.modified = true;
                            self.status = "已将 LLM 输出插入左侧编辑区".to_owned();
                        } else {
                            self.status = "请先在小说编辑面板打开 Markdown 文件".to_owned();
                        }
                    }
                if ui.button("🗑 清空").clicked() {
                    self.llm_prompt.clear();
                    self.llm_output.clear();
                }
            }
        });

        ui.add_space(8.0);
        ui.label("输出结果:");
        egui::ScrollArea::vertical()
            .id_salt("llm_output_scroll")
            .show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut self.llm_output)
                        .desired_width(f32::INFINITY)
                        .desired_rows(12)
                        .hint_text("LLM 输出将显示在这里")
                );
            });
    }

    // ── History tab ───────────────────────────────────────────────────────────

    fn draw_llm_history_tab(&mut self, ui: &mut egui::Ui) {
        // Search box
        ui.horizontal(|ui| {
            ui.label("🔍");
            ui.add(egui::TextEdit::singleline(&mut self.llm_history_search)
                .desired_width(240.0)
                .hint_text("关键词过滤…"));
            if !self.llm_history_search.is_empty() && ui.small_button("✕").clicked() {
                self.llm_history_search.clear();
            }
        });
        ui.add_space(4.0);

        // Clear-all button
        ui.horizontal(|ui| {
            let entry_count = self.llm_history.entries.len();
            ui.label(RichText::new(format!("共 {} 条记录", entry_count)).small()
                .color(Color32::from_gray(160)));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if entry_count > 0 && ui.button("🗑 清空本项目历史").clicked() {
                    self.llm_history_delete_confirm = Some(usize::MAX); // sentinel
                }
            });
        });

        // Confirm clear-all dialog
        if self.llm_history_delete_confirm == Some(usize::MAX) {
            egui::Window::new("确认清空")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ui.ctx(), |ui| {
                    ui.label("确定要清空本项目所有 LLM 历史记录吗？此操作不可撤销。");
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if ui.button("确认清空").clicked() {
                            self.llm_history.entries.clear();
                            if let Some(path) = &self.llm_history_path.clone() {
                                let _ = std::fs::write(path, "{}");
                            }
                            self.llm_history_delete_confirm = None;
                            self.status = "已清空 LLM 历史".to_owned();
                        }
                        if ui.button("取消").clicked() {
                            self.llm_history_delete_confirm = None;
                        }
                    });
                });
            return;
        }

        ui.separator();

        let query = self.llm_history_search.to_lowercase();

        // Collect filtered entries (indices into the full list, newest first)
        let filtered: Vec<usize> = self.llm_history.entries.iter().enumerate()
            .filter(|(_, e)| {
                query.is_empty()
                    || e.prompt.to_lowercase().contains(&query)
                    || e.response.to_lowercase().contains(&query)
            })
            .map(|(i, _)| i)
            .rev()
            .collect();

        if filtered.is_empty() {
            ui.label(RichText::new("暂无历史记录").color(Color32::from_gray(120)));
            return;
        }

        egui::ScrollArea::vertical().id_salt("llm_hist_scroll").show(ui, |ui| {
            // Group by session_key (date portion)
            let mut last_session = String::new();
            let mut to_delete: Option<usize> = None;
            let mut insert_response: Option<String> = None;

            for &idx in &filtered {
                let entry = &self.llm_history.entries[idx];
                let session = &entry.session_key;

                if *session != last_session {
                    ui.add_space(4.0);
                    ui.label(RichText::new(format!("📅 {session}"))
                        .small().color(Color32::from_rgb(120, 180, 240)));
                    last_session = session.clone();
                }

                let expanded = self.llm_history_expanded == Some(idx);
                let prompt_preview: String = entry.prompt.chars().take(60).collect();
                let prompt_label = if entry.prompt.chars().count() > 60 {
                    format!("{prompt_preview}…")
                } else {
                    prompt_preview
                };

                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        // Toggle expand/collapse
                        let toggle = if expanded { "▼" } else { "▶" };
                        if ui.small_button(toggle).clicked() {
                            self.llm_history_expanded = if expanded { None } else { Some(idx) };
                        }
                        ui.label(RichText::new(&prompt_label).small());
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.small_button("🗑").on_hover_text("删除此条").clicked() {
                                to_delete = Some(idx);
                            }
                            if ui.small_button("📋 插入").on_hover_text("将 AI 回复插入当前编辑器").clicked() {
                                insert_response = Some(entry.response.clone());
                            }
                        });
                    });

                    if expanded {
                        ui.add_space(4.0);
                        ui.label(RichText::new("提示词:").small().color(Color32::from_gray(160)));
                        ui.label(RichText::new(&entry.prompt).small().monospace());
                        ui.add_space(4.0);
                        ui.label(RichText::new("回复:").small().color(Color32::from_gray(160)));
                        ui.label(RichText::new(&entry.response).small());
                    }
                });
                ui.add_space(2.0);
            }

            // Apply deferred insert
            if let Some(resp) = insert_response {
                if let Some(lf) = &mut self.left_file {
                    lf.content.push_str("\n\n");
                    lf.content.push_str(&resp);
                    lf.modified = true;
                    self.status = "已将历史回复插入左侧编辑区".to_owned();
                } else {
                    self.status = "请先在小说编辑面板打开 Markdown 文件".to_owned();
                }
            }

            // Apply deferred delete
            if let Some(idx) = to_delete {
                self.llm_history_delete_confirm = Some(idx);
            }
        });

        // Single-entry delete confirmation
        if let Some(idx) = self.llm_history_delete_confirm {
            if idx != usize::MAX {
                egui::Window::new("确认删除")
                    .collapsible(false)
                    .resizable(false)
                    .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                    .show(ui.ctx(), |ui| {
                        ui.label("确定要删除此条历史记录吗？");
                        ui.add_space(8.0);
                        ui.horizontal(|ui| {
                            if ui.button("确认删除").clicked() {
                                if idx < self.llm_history.entries.len() {
                                    self.llm_history.entries.remove(idx);
                                    if let Some(path) = &self.llm_history_path.clone() {
                                        if let Ok(json) = serde_json::to_string_pretty(&self.llm_history) {
                                            let _ = std::fs::write(path, json);
                                        }
                                    }
                                    if self.llm_history_expanded == Some(idx) {
                                        self.llm_history_expanded = None;
                                    }
                                }
                                self.llm_history_delete_confirm = None;
                                self.status = "已删除历史记录".to_owned();
                            }
                            if ui.button("取消").clicked() {
                                self.llm_history_delete_confirm = None;
                            }
                        });
                    });
            }
        }
    }

    /// Build the session key `"<YYYY-MM-DD>"` (ISO 8601, UTC) for grouping
    /// history entries by day.
    fn llm_session_key(&self) -> String {
        let secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        unix_secs_to_iso_date(secs)
    }
}

