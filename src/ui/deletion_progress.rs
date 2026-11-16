//! An isolated progress surface: updates never rebuild the surrounding explorer.
use super::*;
use crate::features::delete::{DeletionPhase, DeletionProgress, DeletionProgressHandle};
use std::time::Instant;

pub(super) struct DeletionProgressView {
    pub handle: DeletionProgressHandle,
    pub snapshot: DeletionProgress,
    pub started: Instant,
    pub elapsed: u64,
    pub path: String,
}

impl DeletionProgressView {
    pub fn poll(&mut self, cx: &mut Context<Self>) {
        let Ok(progress) = self.handle.try_lock() else {
            return;
        };
        let latest = progress.clone();
        drop(progress);
        let elapsed = self.started.elapsed().as_secs();
        if self.snapshot != latest || self.elapsed != elapsed {
            self.snapshot = latest;
            self.elapsed = elapsed;
            cx.notify();
        }
    }
}

impl Render for DeletionProgressView {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let progress = &self.snapshot;
        let removing = progress.phase == DeletionPhase::Removing;
        let title = match progress.phase {
            DeletionPhase::Validating => "正在复核删除目标…",
            DeletionPhase::Removing => "正在彻底删除…",
            DeletionPhase::MovingToTrash => "正在移到系统回收站 / 废纸篓…",
        };
        div()
            .id("deletion-progress")
            .test_support()
            .w_full()
            .p_4()
            .rounded_lg()
            .bg(rgb(0x192632))
            .border_1()
            .border_color(rgb(BORDER))
            .shadow_lg()
            .flex()
            .flex_col()
            .gap_2()
            .text_sm()
            .text_color(rgb(TEXT))
            .child(div().font_weight(FontWeight::SEMIBOLD).child(title))
            .child(
                div()
                    .id("deletion-progress-path")
                    .test_support()
                    .max_h(px(88.0))
                    .overflow_y_scroll()
                    .child(self.path.clone()),
            )
            .when(removing, |el| {
                let total = progress.total.unwrap_or(1).max(1);
                let fraction = (progress.completed as f32 / total as f32).clamp(0., 1.);
                el.child(
                    div().h(px(6.)).rounded_full().bg(rgb(BORDER)).child(
                        div()
                            .h_full()
                            .w(relative(fraction))
                            .rounded_full()
                            .bg(rgb(CYAN)),
                    ),
                )
                .child(format!(
                    "已删除 {} / {} 项（含文件和文件夹）",
                    progress.completed, total
                ))
            })
            .when(!removing, |el| {
                // No partial bar that could be mistaken for a percentage.
                el.child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(div().size(px(7.0)).rounded_full().bg(rgb(AMBER)))
                        .child("处理中"),
                )
            })
            .when(!removing, |el| {
                el.child(div().text_color(rgb(MUTED)).child(
                    if progress.phase == DeletionPhase::MovingToTrash {
                        "系统不提供逐项进度，请等待操作返回。"
                    } else {
                        "正在核对已确认的内容，尚未开始删除。"
                    },
                ))
            })
            .child(format!("已用时 {} 秒", self.elapsed))
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(AMBER))
                    .child("请勿移动、重命名目标或关闭应用。执行中的删除无法安全撤销。"),
            )
    }
}
