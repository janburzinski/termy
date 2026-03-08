# Termy Go bindings

Go SDK for Termy plugins that speak newline-delimited JSON over stdio.

## What this package provides

- Protocol constants and manifest/message types aligned with `termy_plugin_core`
- `PluginSession` handshake and message loop helpers
- Convenience methods for `log`, `toast`, and `pong`

## Quick start

```go
package main

import (
	"log"

	termybindings "github.com/lassejlv/termy/bindings/go-bindings"
)

func main() {
	session, err := termybindings.NewStdioSession(termybindings.PluginMetadata{
		PluginID: "example.hello",
		Name:     "Hello Plugin",
		Version:  "0.1.0",
	})
	if err != nil {
		log.Fatal(err)
	}

	_ = session.RunUntilShutdown(func(message termybindings.HostRPCMessage, current *termybindings.PluginSession) error {
		switch message.Type {
		case "ping":
			return current.SendPong()
		case "invoke_command":
			return current.SendLog(termybindings.PluginLogLevelInfo, "invoke received")
		}
		return nil
	})
}
```

## Run tests

```bash
go test ./...
```
