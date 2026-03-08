package main

import (
	"log"

	termybindings "github.com/lassejlv/termy/bindings/go-bindings"
)

func main() {
	session, err := termybindings.NewStdioSession(termybindings.PluginMetadata{
		PluginID: "example.go.hello",
		Name:     "Hello Plugin (Go)",
		Version:  "0.1.0",
		Capabilities: []termybindings.PluginCapability{
			termybindings.PluginCapabilityCommandProvider,
		},
	})
	if err != nil {
		log.Fatal(err)
	}

	err = session.RunUntilShutdown(func(message termybindings.HostRPCMessage, current *termybindings.PluginSession) error {
		switch message.Type {
		case "ping":
			return current.SendPong()
		case "invoke_command":
			commandID, ok := termybindings.CommandID(message)
			if ok {
				_ = current.SendLog(termybindings.PluginLogLevelInfo, "invoke command: "+commandID)
				return current.SendToast(termybindings.PluginToastLevelSuccess, "Hello from Go plugin", nil)
			}
		}
		return nil
	})
	if err != nil {
		log.Fatal(err)
	}
}
