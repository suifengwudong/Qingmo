use std::sync::Arc;
use egui::{RichText, Color32};
use super::super::{TextToolApp, LlmBackend, LlmTask, MockBackend, ApiBackend};

impl TextToolApp {
    // ── Panel: LLM Assistance ─────────────────────────────────────────────────

    pub(in crate::app) fn draw_llm_panel(&mut self, ctx: &egui::Context) {
        // Poll for completed background task each frame
        if let Some(task) = &self.llm_task {
            match task.receiver.try_recv() {
                Ok(Ok(text)) => {
                    self.llm_output = text;
                    self.status = "LLM 补全完成".to_owned();
                    self.llm_task = None;
                    ctx.request_repaint();
                }
                Ok(Err(e)) => {
                    self.llm_output = format!("【错误】{e}");
                    self.status = format!("LLM 调用失败: {e}");
                    self.llm_task = None;
                    ctx.request_repaint();
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    // Still running – keep repainting so the spinner stays animated
                    ctx.request_repaint();
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.llm_output = "【错误】后台线程意外断开".to_owned();
                    self.llm_task = None;
                }
            }
        }

        let is_running = self.llm_task.is_some();

        egui::SidePanel::left("llm_config")
            .resizable(true)
            .default_width(240.0)
            .min_width(160.0)
            .show(ctx, |ui| {
                ui.add_space(4.0);
                ui.heading("LLM 配置");
                ui.separator();

                // Backend selector
                ui.label("接口类型:");
                ui.horizontal(|ui| {
                    if ui.selectable_label(self.llm_backend_idx == 0, "🧪 模拟模型").clicked() {
                        self.llm_backend_idx = 0;
                    }
                    if ui.selectable_label(self.llm_backend_idx == 1, "🌐 HTTP API").clicked() {
                        self.llm_backend_idx = 1;
                    }
                });
                ui.add_space(4.0);
                ui.separator();

                if self.llm_backend_idx == 1 {
                    // API backend config
                    ui.checkbox(&mut self.llm_config.use_local, "本地模型 (Ollama)");
                    ui.add_space(4.0);
                    if self.llm_config.use_local {
                        ui.label("模型名称 / 路径:");
                        ui.text_edit_singleline(&mut self.llm_config.model_path)
                            .on_hover_text("Ollama 模型名称，如 llama2、phi 等");
                        ui.add_space(4.0);
                        ui.label("API 地址:");
                        ui.text_edit_singleline(&mut self.llm_config.api_url)
                            .on_hover_text("Ollama 端点，如 http://localhost:11434/api/generate");
                    } else {
                        ui.label("API 地址 (OpenAI 兼容):");
                        ui.text_edit_singleline(&mut self.llm_config.api_url)
                            .on_hover_text("如 https://api.openai.com/v1/chat/completions");
                        ui.add_space(4.0);
                        ui.label("模型名称:");
                        ui.text_edit_singleline(&mut self.llm_config.model_path)
                            .on_hover_text("如 gpt-4o、gpt-3.5-turbo 等");
                    }
                } else {
                    ui.label(
                        RichText::new("使用内置模拟模型，\n无需配置。")
                            .color(Color32::from_gray(150))
                            .small(),
                    );
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
                    RichText::new("支持模型:\nOllama (llama2, phi…)\nOpenAI 兼容 API\n模拟模式 (无需网络)")
                        .color(Color32::from_gray(140))
                        .small(),
                );
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("LLM 辅助写作");
            ui.separator();

            // ── Structured context injection ───────────────────────────────────
            ui.label(RichText::new("注入结构化上下文 (追加到提示词末尾):").small()
                .color(Color32::from_gray(160)));
            ui.horizontal(|ui| {
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

            ui.add_space(6.0);
            ui.label("提示词 / 上下文:");
            egui::ScrollArea::vertical()
                .id_salt("llm_prompt_scroll")
                .max_height(200.0)
                .show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut self.llm_prompt)
                            .desired_width(f32::INFINITY)
                            .desired_rows(8)
                            .hint_text("输入提示词，例如：\n续写以下场景：\n或 优化以下对话：")
                    );
                });

            ui.add_space(4.0);
            ui.horizontal(|ui| {
                if is_running {
                    ui.add(egui::Spinner::new());
                    ui.label(RichText::new("正在调用 LLM…").color(Color32::from_rgb(200, 200, 80)));
                    if ui.button("⏹ 取消").clicked() {
                        // Drop the task to abandon the channel (thread finishes naturally)
                        self.llm_task = None;
                        self.status = "已取消 LLM 调用".to_owned();
                    }
                } else {
                    if ui.button("▶ 调用 LLM 补全").clicked() {
                        let backend: Arc<dyn LlmBackend> = if self.llm_backend_idx == 1 {
                            Arc::new(ApiBackend)
                        } else {
                            Arc::new(MockBackend)
                        };
                        self.llm_task = Some(LlmTask::spawn(
                            backend,
                            self.llm_config.clone(),
                            self.llm_prompt.clone(),
                        ));
                        self.status = "LLM 调用已提交，后台处理中…".to_owned();
                    }
                    if ui.button("插入到左侧编辑区").clicked() {
                        if !self.llm_output.is_empty() {
                            if let Some(lf) = &mut self.left_file {
                                lf.content.push_str("\n\n");
                                lf.content.push_str(&self.llm_output);
                                lf.modified = true;
                                self.status = "已将 LLM 输出插入左侧编辑区".to_owned();
                            } else {
                                self.status = "请先在小说编辑面板打开 Markdown 文件".to_owned();
                            }
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
        });
    }
}

// Keep a thin `llm_simulate` shim on TextToolApp so existing callers (if any) still compile.
impl TextToolApp {
    #[allow(dead_code)]
    pub(in crate::app) fn llm_simulate(&self) -> String {
        let backend = MockBackend;
        backend.complete(&self.llm_config, &self.llm_prompt)
            .unwrap_or_else(|e| e)
    }
}

