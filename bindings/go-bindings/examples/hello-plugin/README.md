# Hello Plugin (Go)

Minimal example plugin using `github.com/lassejlv/termy/bindings/go-bindings`.

## Build

```bash
go build -o hello-plugin .
```

## Install in Termy

Copy this folder into your plugin directory:

```bash
cp -R . ~/.config/termy/plugins/example-go-hello
```

Then open `Settings -> Plugins` and click `Reload`.
