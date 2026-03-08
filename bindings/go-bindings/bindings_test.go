package termybindings

import (
	"bufio"
	"bytes"
	"errors"
	"strings"
	"testing"
)

func TestNewSessionSendsPluginHello(t *testing.T) {
	input := strings.NewReader("{\"type\":\"hello\",\"payload\":{\"protocol_version\":1,\"host_name\":\"termy\",\"host_version\":\"0.1.0\",\"plugin_id\":\"example.hello\"}}\n")
	reader := bufio.NewReader(input)
	var output bytes.Buffer

	session, err := NewSession(reader, &output, PluginMetadata{
		PluginID: "example.hello",
		Name:     "Hello Plugin",
		Version:  "0.1.0",
	})
	if err != nil {
		t.Fatalf("new session failed: %v", err)
	}
	if session.PluginID != "example.hello" {
		t.Fatalf("unexpected plugin id: %s", session.PluginID)
	}

	got := output.String()
	if !strings.Contains(got, "\"type\":\"hello\"") {
		t.Fatalf("expected plugin hello message, got: %s", got)
	}
}

func TestNewSessionRejectsPluginIDMismatch(t *testing.T) {
	input := strings.NewReader("{\"type\":\"hello\",\"payload\":{\"protocol_version\":1,\"host_name\":\"termy\",\"host_version\":\"0.1.0\",\"plugin_id\":\"wrong.id\"}}\n")
	reader := bufio.NewReader(input)
	var output bytes.Buffer

	_, err := NewSession(reader, &output, PluginMetadata{
		PluginID: "example.hello",
		Name:     "Hello Plugin",
		Version:  "0.1.0",
	})
	var mismatchErr PluginIDMismatchError
	if !errors.As(err, &mismatchErr) {
		t.Fatalf("expected PluginIDMismatchError, got: %v", err)
	}
}

func TestRunUntilShutdown(t *testing.T) {
	input := strings.NewReader(
		"{\"type\":\"hello\",\"payload\":{\"protocol_version\":1,\"host_name\":\"termy\",\"host_version\":\"0.1.0\",\"plugin_id\":\"example.hello\"}}\n" +
			"{\"type\":\"ping\"}\n" +
			"{\"type\":\"shutdown\"}\n",
	)
	reader := bufio.NewReader(input)
	var output bytes.Buffer

	session, err := NewSession(reader, &output, PluginMetadata{
		PluginID: "example.hello",
		Name:     "Hello Plugin",
		Version:  "0.1.0",
	})
	if err != nil {
		t.Fatalf("new session failed: %v", err)
	}

	var seen []string
	err = session.RunUntilShutdown(func(message HostRPCMessage, current *PluginSession) error {
		seen = append(seen, message.Type)
		if message.Type == "ping" {
			return current.SendPong()
		}
		return nil
	})
	if err != nil {
		t.Fatalf("run loop failed: %v", err)
	}
	if len(seen) != 2 || seen[0] != "ping" || seen[1] != "shutdown" {
		t.Fatalf("unexpected messages: %#v", seen)
	}
	if !strings.Contains(output.String(), "\"type\":\"pong\"") {
		t.Fatalf("expected pong in output, got: %s", output.String())
	}
}
