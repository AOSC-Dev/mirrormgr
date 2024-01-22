# lists
branch = 分支：{$branch}
component = 组件：{$comp}
mirror = 镜像源：{$mirror}
custom = [自定义]

# messages
set-branch = 已将 {$branch} 设置为默认分支
mirror-list-explain = 行头的 '*' 或高亮代表镜像源已启用：
test-mirrors = 正在测试镜像源性能 ...
test-mirrors-sync = 正在测试镜像源性能 ({$count}/{$all}) ...
set-fastest-mirror = 最快的镜像源为：{$mirror}，下载速率：{$speed}，现将 {$mirror} 设为默认镜像源 ...
enable-comp = 正在启用 {$comp} 组件 ...
disable-comp = 正在禁用 {$comp} 组件 ...
set-mirror = 正在将 {$mirror} 设定为默认镜像源！
add-mirror = 正在将 {$mirror} 的镜像源信息写入 sources.list ...
add-custom-mirror = 正在将 {$mirror} 的自定义镜像源信息写入 {$path} ...
remove-mirror = 正在从 sources.list 移除 {$mirror} 的镜像源信息 ...
remove-custom-mirror = 正在从 {$path} 移除 {$mirror} 的自定义镜像源信息 ...
write-status = 正在写入 apt-gen-list 状态文件 ...
write-sources = 正在生成 /etc/apt/sources.list ...
write-omakase-config = 正在生成 /etc/omakase/config.toml ...
run-refresh = 正在刷新镜像源 ...
trying-get-mirror = 正在尝试访问镜像源 ...
fastest-mirror = 最快的镜像源为：{$mirror}，下载速率：{$speed}
order-mirror = 为已启用的镜像源排序
activating-count-mirrors = 正在启用 {$count} 个镜像源
select-open-or-close-mirrors = 选中镜像源以启用或禁用
help-message = 按 [Space] 或 [Enter] 启用和禁用镜像源，按 [Esc] 应用更改，按 [Ctrl-c] 退出。

# error messages
comp-not-enabled = 组件 {$comp} 未启用或不存在。
comp-not-found = 组件 {$comp} 不存在。
comp-already-enabled = 组件 {$comp} 已启用。
branch-not-found = 分支未定义或不存在！
branch-data-error = 无法读取分支列表数据！
branch-already-enabled = 分支 {$branch} 已经启动
mirror-not-found = 找不到镜像源：{$mirror} 。请使用 `oma mirror' 或 `mirrormgr' 命令查看源列表并选择源，或使用 `oma mirror custom-mirror' 添加自定义源。
mirror-already-enabled = 镜像源 {$mirror} 之前已被启用！
mirror-already-disabled = 镜像源 {$mirror} 之前已被关闭或不存在 ！
mirror-error = 无法从 {$mirror} 下载测试数据，请检查你的网络连接！
mirror-test-failed = 无法测试镜像源，请检查你的网络连接！
custom-mirror-not-found = 自定义镜像源 {$mirror} 不存在！
custom-mirror-already-exist = 自定义镜像源 {$mirror} 已存在！
custom-mirror-not-url = 您指定的镜像源 URL {$mirror_url} 不是合法 URL ！
custom-mirror-name-error = 您指定的镜像源 {$mirror_name} 未在镜像源数据文件中定义！
no-delete-only-mirror = 无法移除唯一启用的镜像源！
no-delete-only-comp = 不允许删除必要组件 "main" 。
status-file-not-found = 无法创建状态文件 ({$path})，请提权后运行 `oma mirror' 或 `mirrormgr' 以修正该问题。
status-file-read-error = 状态文件格式过老或已损坏，请提权后运行 `oma mirror' 或 `mirrormgr' 以修正该问题。
download-mirror-metadata-failed = 无法从自定义软件源下载元数据：您的软件源配置信息可能有误。
execute-pkexec-fail = 无法执行 `pkexec' 命令：{$e}。

# file content
generated = # 本文件使用 mirrormgr 生成，请勿编辑！
