complete -c mirrormgr -n "__fish_use_subcommand" -s d -l debug
complete -c mirrormgr -n "__fish_use_subcommand" -s h -l help -d 'Print help'
complete -c mirrormgr -n "__fish_use_subcommand" -s V -l version -d 'Print version'
complete -c mirrormgr -n "__fish_use_subcommand" -f -a "set" -d 'Set APT repository mirror, branch and components'
complete -c mirrormgr -n "__fish_use_subcommand" -f -a "add" -d 'Add APT repository mirror, branch and components'
complete -c mirrormgr -n "__fish_use_subcommand" -f -a "remove" -d 'Remove APT repository mirror, branch and components'
complete -c mirrormgr -n "__fish_use_subcommand" -f -a "reset" -d 'Reset all APT repositories mirror settings'
complete -c mirrormgr -n "__fish_use_subcommand" -f -a "menu" -d 'Mirrormgr menu'
complete -c mirrormgr -n "__fish_use_subcommand" -f -a "speedtest" -d 'Speedtest mirrors'
complete -c mirrormgr -n "__fish_use_subcommand" -f -a "custom-mirrors" -d 'Edit custom mirror settings'
complete -c mirrormgr -n "__fish_use_subcommand" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c mirrormgr -n "__fish_seen_subcommand_from set" -s m -l mirror -r
complete -c mirrormgr -n "__fish_seen_subcommand_from set" -s b -l branch -r
complete -c mirrormgr -n "__fish_seen_subcommand_from set" -s h -l help -d 'Print help'
complete -c mirrormgr -n "__fish_seen_subcommand_from add" -s m -l mirrors -r
complete -c mirrormgr -n "__fish_seen_subcommand_from add" -s c -l components -r
complete -c mirrormgr -n "__fish_seen_subcommand_from add" -s h -l help -d 'Print help'
complete -c mirrormgr -n "__fish_seen_subcommand_from remove" -s m -l mirrors -r
complete -c mirrormgr -n "__fish_seen_subcommand_from remove" -s c -l components -r
complete -c mirrormgr -n "__fish_seen_subcommand_from remove" -s h -l help -d 'Print help'
complete -c mirrormgr -n "__fish_seen_subcommand_from reset" -s h -l help -d 'Print help'
complete -c mirrormgr -n "__fish_seen_subcommand_from menu" -s h -l help -d 'Print help'
complete -c mirrormgr -n "__fish_seen_subcommand_from speedtest" -s h -l help -d 'Print help'
complete -c mirrormgr -n "__fish_seen_subcommand_from custom-mirrors" -s h -l help -d 'Print help'
complete -c mirrormgr -n "__fish_seen_subcommand_from help; and not __fish_seen_subcommand_from set; and not __fish_seen_subcommand_from add; and not __fish_seen_subcommand_from remove; and not __fish_seen_subcommand_from reset; and not __fish_seen_subcommand_from menu; and not __fish_seen_subcommand_from speedtest; and not __fish_seen_subcommand_from custom-mirrors; and not __fish_seen_subcommand_from help" -f -a "set" -d 'Set APT repository mirror, branch and components'
complete -c mirrormgr -n "__fish_seen_subcommand_from help; and not __fish_seen_subcommand_from set; and not __fish_seen_subcommand_from add; and not __fish_seen_subcommand_from remove; and not __fish_seen_subcommand_from reset; and not __fish_seen_subcommand_from menu; and not __fish_seen_subcommand_from speedtest; and not __fish_seen_subcommand_from custom-mirrors; and not __fish_seen_subcommand_from help" -f -a "add" -d 'Add APT repository mirror, branch and components'
complete -c mirrormgr -n "__fish_seen_subcommand_from help; and not __fish_seen_subcommand_from set; and not __fish_seen_subcommand_from add; and not __fish_seen_subcommand_from remove; and not __fish_seen_subcommand_from reset; and not __fish_seen_subcommand_from menu; and not __fish_seen_subcommand_from speedtest; and not __fish_seen_subcommand_from custom-mirrors; and not __fish_seen_subcommand_from help" -f -a "remove" -d 'Remove APT repository mirror, branch and components'
complete -c mirrormgr -n "__fish_seen_subcommand_from help; and not __fish_seen_subcommand_from set; and not __fish_seen_subcommand_from add; and not __fish_seen_subcommand_from remove; and not __fish_seen_subcommand_from reset; and not __fish_seen_subcommand_from menu; and not __fish_seen_subcommand_from speedtest; and not __fish_seen_subcommand_from custom-mirrors; and not __fish_seen_subcommand_from help" -f -a "reset" -d 'Reset all APT repositories mirror settings'
complete -c mirrormgr -n "__fish_seen_subcommand_from help; and not __fish_seen_subcommand_from set; and not __fish_seen_subcommand_from add; and not __fish_seen_subcommand_from remove; and not __fish_seen_subcommand_from reset; and not __fish_seen_subcommand_from menu; and not __fish_seen_subcommand_from speedtest; and not __fish_seen_subcommand_from custom-mirrors; and not __fish_seen_subcommand_from help" -f -a "menu" -d 'Mirrormgr menu'
complete -c mirrormgr -n "__fish_seen_subcommand_from help; and not __fish_seen_subcommand_from set; and not __fish_seen_subcommand_from add; and not __fish_seen_subcommand_from remove; and not __fish_seen_subcommand_from reset; and not __fish_seen_subcommand_from menu; and not __fish_seen_subcommand_from speedtest; and not __fish_seen_subcommand_from custom-mirrors; and not __fish_seen_subcommand_from help" -f -a "speedtest" -d 'Speedtest mirrors'
complete -c mirrormgr -n "__fish_seen_subcommand_from help; and not __fish_seen_subcommand_from set; and not __fish_seen_subcommand_from add; and not __fish_seen_subcommand_from remove; and not __fish_seen_subcommand_from reset; and not __fish_seen_subcommand_from menu; and not __fish_seen_subcommand_from speedtest; and not __fish_seen_subcommand_from custom-mirrors; and not __fish_seen_subcommand_from help" -f -a "custom-mirrors" -d 'Edit custom mirror settings'
complete -c mirrormgr -n "__fish_seen_subcommand_from help; and not __fish_seen_subcommand_from set; and not __fish_seen_subcommand_from add; and not __fish_seen_subcommand_from remove; and not __fish_seen_subcommand_from reset; and not __fish_seen_subcommand_from menu; and not __fish_seen_subcommand_from speedtest; and not __fish_seen_subcommand_from custom-mirrors; and not __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
