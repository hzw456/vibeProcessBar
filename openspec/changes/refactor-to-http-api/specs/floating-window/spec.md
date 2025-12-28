## ADDED Requirements

### Requirement: 悬浮窗显示

系统 SHALL 提供一个可配置的悬浮窗，用于实时显示 AI 代码生成进度。

悬浮窗 SHALL 支持透明背景和置顶显示。

悬浮窗 SHALL 占用最小屏幕空间（默认 200x80 像素）。

#### Scenario: 启动时显示悬浮窗

- **WHEN** 用户启动应用程序
- **THEN** 系统显示悬浮窗在屏幕默认位置
- **AND** 悬浮窗显示初始状态（"准备就绪"）
- **AND** 悬浮窗显示 0 个 token

#### Scenario: 悬浮窗透明度和位置配置

- **WHEN** 用户在设置中调整透明度
- **THEN** 悬浮窗背景透明度实时更新
- **AND** 用户可以拖拽悬浮窗到任意位置
- **AND** 系统记住最后位置并在下次启动时恢复

#### Scenario: 悬浮窗主题切换

- **WHEN** 用户切换主题（浅色/深色/自定义）
- **THEN** 悬浮窗颜色方案实时更新
- **AND** 所有 UI 组件使用新主题颜色

### Requirement: Token 数量显示

系统 SHALL 在悬浮窗中显示当前任务的 token 消耗。

Token 数量 SHALL 实时更新。

#### Scenario: 显示 token 数量

- **WHEN** 外部工具通过 HTTP API 更新 token
- **THEN** 悬浮窗实时显示当前 token 数量
- **AND** Token 数量格式化为 K、M 单位（如 1.5K、2.3M）

#### Scenario: Token 计数器

- **WHEN** 任务进行中
- **THEN** 悬浮窗显示累计 token 消耗
- **AND** Token 数量随每个 API 更新递增
- **AND** 任务完成时显示最终 token 数

### Requirement: 进度状态显示

系统 SHALL 显示当前任务的进度和状态。

#### Scenario: 进行中状态

- **WHEN** 任务状态为 running
- **THEN** 悬浮窗显示进度条动画
- **AND** 状态文本显示"进行中"
- **AND** 进度百分比实时更新

#### Scenario: 已完成状态

- **WHEN** 任务状态为 completed
- **THEN** 悬浮窗显示完成状态
- **AND** 进度条显示 100%
- **AND** 状态文本显示"已完成"
- **AND** 3 秒后自动重置或保持显示

#### Scenario: 错误状态

- **WHEN** 任务状态为 error
- **THEN** 悬浮窗显示错误状态
- **AND** 状态文本显示"错误"
- **AND** 显示错误提示

### Requirement: 窗口跳转

系统 SHALL 支持点击悬浮窗激活对应的 IDE 窗口。

#### Scenario: 点击悬浮窗跳转

- **WHEN** 用户左键点击悬浮窗
- **THEN** 系统激活任务关联的 IDE 窗口
- **AND** 如果窗口已最小化，则恢复窗口
- **AND** 如果窗口在后台，则切换到前台

#### Scenario: 多任务窗口切换

- **WHEN** 悬浮窗显示多个任务
- **AND** 用户点击悬浮窗
- **THEN** 系统激活当前选中任务的 IDE 窗口
- **AND** 支持通过快捷键或菜单切换不同任务的窗口

### Requirement: 悬浮窗交互

系统 SHALL 提供丰富的悬浮窗交互功能。

#### Scenario: 右键菜单

- **WHEN** 用户右键点击悬浮窗
- **THEN** 显示上下文菜单
- **AND** 菜单包含：设置、重置任务、位置预设、关于

#### Scenario: 滚轮调节进度

- **WHEN** 用户滚动鼠标滚轮
- **THEN** 当前任务进度增加或减少 5%
- **AND** 进度值范围为 0-100

#### Scenario: 全局快捷键

- **WHEN** 用户按下 Alt+P（默认快捷键）
- **THEN** 悬浮窗显示/隐藏切换
- **AND** 快捷键可在设置中自定义

### Requirement: 系统集成

系统 SHALL 与操作系统深度集成，提供开机自启动和系统托盘支持。

#### Scenario: 开机自启动

- **WHEN** 用户启用开机自启动
- **THEN** 系统在用户登录后自动启动
- **AND** 悬浮窗显示在最后记录的位置
- **AND** 自启动可在设置中禁用

#### Scenario: 系统托盘集成

- **WHEN** 用户最小化或隐藏悬浮窗
- **THEN** 系统显示托盘图标
- **AND** 点击托盘图标显示悬浮窗
- **AND** 托盘图标显示当前进度状态
