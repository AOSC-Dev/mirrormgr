# lists
branch = 分支：{$branch}
component = 组件：{$comp}
mirror = 镜像源：{$mirror}
custom = [自定义]

# messages
set-branch = 已将 {$branch} 设置为默认分支
mirror-list-explain = 行头的 '*' 或高亮代表正在使用该镜像源：
test-mirrors = 正在测试镜像源性能 ...
test-mirrors-sync = 正在测试镜像源性能 ({$count}/{$all}) ...
set-fastest-mirror = 最快的镜像源为：{$mirror}，速率：{$speed}，现将 {$mirror} 设置为默认镜像源 ...
enable-comp = 正在启用 {$comp} 组件 ...
disable-comp = 正在禁用 {$comp} 组件 ...
set-mirror = 正在将 {$mirror} 设定为镜像源！
add-mirror = 正在将 {$mirror} 的镜像源信息写入 sources.list ...
add-custom-mirror = 正在将 {$mirror} 的自定义镜像源信息写入 {$path}
remove-mirror = 正在从 sources.list 移除 {$mirror} 的镜像源信息 ...
remove-custom-mirror = 正在从 {$path} 移除 {$mirror} 的自定义镜像源信息
write-status = 正在写入 apt-gen-list 状态文件 ...
write-sources = 正在生成 /etc/apt/sources.list ...
run-apt = 正在运行 `apt-get update` ...
run-atm-refresh = 正在运行 `atm refresh` ...
trying-get-mirror = 正在尝试访问源 ...


# error messages
comp-not-enabled = 组件 {$comp} 未启用或不存在。
comp-not-found = 组件 {$comp} 不存在。
comp-already-enabled = 组件 {$comp} 已启用。
branch-not-found = 分支未定义或不存在！
branch-data-error = 无法读取分支列表数据！
mirror-not-found = 找不到镜像源：{$mirror} 。
mirror-already-enabled = 之前已启用 {$mirror} ！
mirror-error = 无法从 {$mirror} 下载测试数据，请检查你的网络连接！
mirror-test-failed = 无法测试任何镜像源！请检查你的网络连接！
custom-mirror-not-found = 自定义镜像源 {$mirror} 不存在！
custom-mirror-already-exist = 自定义镜像源 {$mirror} 已存在！
custom-mirror-not-url = mirror_url 不是合法 URL ！
custom-mirror-name-error = mirror_name 未在镜像源数据文件中定义！
no-delete-only-mirror = 无法移除唯一启用的镜像源！
no-delete-only-comp = 程序已拒绝删除必需组件 "main" 。
status-file-not-found = 状态文件 ({$path}) 不存在！请用 root 用户运行 apt-gen-list 以创建状态文件！
status-file-read-error = 状态文件格式过老或已损坏，请用 root 用户运行该命令以修正状态文件！
debs-path-in-url = pt-gen-list 发现您的自定义 URL 结尾发现 '/debs' 字段，这是配置自定义软件源时的一大常见错误。请删去此节后重试。
download-mirror-metadata-failed = 从自定义软件源元数据下载失败：您的软件源配置信息可能不正确。

# file content
generated = # 本文件使用 apt-gen-list 生成，请勿编辑！
