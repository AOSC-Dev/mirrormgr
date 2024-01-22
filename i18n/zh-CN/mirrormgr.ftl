# lists

# messages
disable-comp = 正在禁用 {$comp} 组件 ...
set-mirror = 正在将 {$mirror} 设定为默认镜像源！
remove-mirror = 正在从 sources.list 移除 {$mirror} 的镜像源信息 ...
write-sources = 正在生成 /etc/apt/sources.list ...
run-refresh = 正在刷新镜像源 ...
activating-count-mirrors = 正在启用 {$count} 个镜像源
select-open-or-close-mirrors = 选中镜像源以启用或禁用
help-message = 按 [Space] 或 [Enter] 启用和禁用镜像源，按 [Esc] 应用更改，按 [Ctrl-c] 退出。

# error messages
comp-not-found = 组件 {$comp} 不存在。
comp-already-enabled = 组件 {$comp} 已启用。
branch-not-found = 分支未定义或不存在！
branch-already-enabled = 分支 {$branch} 已经启动
mirror-not-found = 找不到镜像源：{$mirror} 。请使用 `oma mirror' 或 `mirrormgr' 命令查看源列表并选择源，或使用 `oma mirror custom-mirror' 添加自定义源。
mirror-already-enabled = 镜像源 {$mirror} 之前已被启用！
mirror-already-disabled = 镜像源 {$mirror} 之前已被关闭或不存在 ！
mirror-error = 无法从 {$mirror} 下载测试数据，请检查你的网络连接！
no-delete-only-mirror = 无法移除唯一启用的镜像源！
no-delete-only-comp = 不允许删除必要组件 "main" 。
execute-pkexec-fail = 无法执行 `pkexec' 命令：{$e}。

# file content
generated = # 本文件使用 mirrormgr 生成，请勿编辑！
