# Control Zone 

## czctrl

a command line to for mnanaging control zone

```
Usage: czctrl [OPTIONS] <COMMAND>

Commands:
  apply    Apply Control Zone from Yaml
  down     Down Control Zone from Yaml
  list     List Control Zones
  observe  Monitor Control Zone
  conn     Connect to Control Zone
  create   Create Control Zone
  start    Start Control Zone
  update   Update Control ZOne
  stop     Stop Control Zone
  remove   Remove Control Zone
  inspect  Inspect Control Zone
  log      Log From Control Zone
  pod      Manage Pod of Control Zone
  help     Print this message or the help of the given subcommand(s)

Options:
  -d, --dry-run              just print the results
      --root <ROOT>          
      --vruntime <VRUNTIME>  [default: libvirt] [possible values: libvirt, qemu]
  -h, --help                 Print help
  -V, --version              Print version
```
managing pod in control zone

```
Usage: czctrl pod [OPTIONS] <COMMAND>

Commands:
  add     Add Pod to Control Zone
  delete  Delete Pod from Control Zone
  show    List Pod of Control Zone
  help    Print this message or the help of the given subcommand(s)

Options:
  -d, --dry-run              just print the results
      --root <ROOT>          
      --vruntime <VRUNTIME>  [default: libvirt] [possible values: libvirt, qemu]
  -h, --help                 Print help
  -V, --version              Print version
```

## czdaemon

daemon ran in control zone virtual machine, listening for command and fetch vm infos

## chsd

change scheduler tool in virtual machine, to manage scheduler of pods
