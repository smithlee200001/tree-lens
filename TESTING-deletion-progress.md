# 删除进度界面

- 基线：main `d18275f57d1de28bdba0e304e9bf76da7295a33e`。
- 确认后的删除进度使用独立 GPUI Entity 浮层，不移动列表、不遮暗其他文字、不打开/关闭全窗口模态弹窗。右下角显示完整可滚动路径、当前阶段、实际已删除项目数和耗时。
- 彻底删除沿用已确认子树和全部路径/身份/挂载点校验，只有原有删除调用成功才递增计数；分母来自现有确认快照（包含选中根目录），没有额外预扫描、定期目录扫描或另写不安全递归删除。
- 系统回收站没有逐项回调，显示不定进度与耗时，不伪造百分比。复核阶段也不显示错误的删除百分比。
- 工作线程通过单个最新进度快照通信，不为每个文件堆积事件；80 ms 主轮询只通知进度 Entity，界面轮询使用 try_lock，不阻塞等待工作线程。
- 删除失败保留已完成计数；成功、失败、线程异常均沿用只读后台核对和保留视图逻辑，不重复删除。
- 不提供误导性的取消按钮；确认前仍可取消，开始后不能保证安全中断或恢复。请不要关闭应用或并发修改目标路径。

## 检查

- `cargo fmt --all -- --check`、`git diff --check`、`cargo clippy --locked --features ui-tests --all-targets -- -D warnings`：通过。
- `cargo test --locked --no-default-features --lib`：89 passed，3 个既有集成测试忽略。新增：递归准确计数、确认后内容变化不报告已删除、回收失败无伪造总数或删除回退。
- `cargo test --locked --features ui-tests --all-targets`：114 个库测试、3 个架构测试通过，5 个既有专用集成测试默认忽略。
- 显式运行 `deletion_text_pixels_are_identical_across_frames -- --ignored`：软件 Vulkan 离屏渲染通过，现有完整标题/侧栏逐像素一致性断言未修改。
- 显式隔离 GIO 回收站集成测试 2 项通过（仅自身合成 HOME/Trash）。Unix socket 专用测试因沙箱禁止 bind（EPERM）未能运行到断言，不是扫描结果失败。
- GUI 自动化新增：最小窗口浮层可见、路径可见、不改变导航控件边界、进度实体通知、锁占用不阻塞 UI，以及确认执行/完成后的浮层生命周期。
- 原生 macOS / Windows 的大文件夹速度和视觉体验需对应平台实测，不能用 Linux 测试宣称已验证。
- `ui_snapshot -- <synthetic-directory> <output.png> 1440 920 --delete-progress` 为纯展示夹具（不启动删除），示例数字不是性能测量。
