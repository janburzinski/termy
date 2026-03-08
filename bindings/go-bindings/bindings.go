package termybindings

import (
	"bufio"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"os"
)

const (
	PluginManifestFileName = "termy-plugin.json"
	PluginProtocolVersion  = 1
)

type PluginRuntime string

const (
	PluginRuntimeExecutable PluginRuntime = "executable"
)

type PluginPermission string

const (
	PluginPermissionFilesystemRead  PluginPermission = "filesystem_read"
	PluginPermissionFilesystemWrite PluginPermission = "filesystem_write"
	PluginPermissionNetwork         PluginPermission = "network"
	PluginPermissionShell           PluginPermission = "shell"
	PluginPermissionClipboard       PluginPermission = "clipboard"
	PluginPermissionNotifications   PluginPermission = "notifications"
	PluginPermissionTerminalRead    PluginPermission = "terminal_read"
	PluginPermissionTerminalWrite   PluginPermission = "terminal_write"
	PluginPermissionUIPanels        PluginPermission = "ui_panels"
)

type PluginContributions struct {
	Commands []PluginCommandContribution `json:"commands,omitempty"`
}

type PluginCommandContribution struct {
	ID          string  `json:"id"`
	Title       string  `json:"title"`
	Description *string `json:"description,omitempty"`
}

type PluginManifest struct {
	SchemaVersion      int                 `json:"schema_version"`
	ID                 string              `json:"id"`
	Name               string              `json:"name"`
	Version            string              `json:"version"`
	Description        *string             `json:"description,omitempty"`
	Author             *string             `json:"author,omitempty"`
	MinimumHostVersion *string             `json:"minimum_host_version,omitempty"`
	APIVersion         *int                `json:"api_version,omitempty"`
	Runtime            PluginRuntime       `json:"runtime,omitempty"`
	Entrypoint         string              `json:"entrypoint"`
	Autostart          *bool               `json:"autostart,omitempty"`
	Permissions        []PluginPermission  `json:"permissions,omitempty"`
	Contributes        PluginContributions `json:"contributes,omitempty"`
}

type HostHello struct {
	ProtocolVersion int    `json:"protocol_version"`
	HostName        string `json:"host_name"`
	HostVersion     string `json:"host_version"`
	PluginID        string `json:"plugin_id"`
}

type HostCommandInvocation struct {
	CommandID string `json:"command_id"`
}

type HostRPCMessage struct {
	Type    string          `json:"type"`
	Payload json.RawMessage `json:"payload,omitempty"`
}

func (m HostRPCMessage) AsHello() (HostHello, error) {
	if m.Type != "hello" {
		return HostHello{}, fmt.Errorf("expected hello, got %q", m.Type)
	}
	var payload HostHello
	if err := json.Unmarshal(m.Payload, &payload); err != nil {
		return HostHello{}, err
	}
	return payload, nil
}

func (m HostRPCMessage) AsInvokeCommand() (HostCommandInvocation, error) {
	if m.Type != "invoke_command" {
		return HostCommandInvocation{}, fmt.Errorf("expected invoke_command, got %q", m.Type)
	}
	var payload HostCommandInvocation
	if err := json.Unmarshal(m.Payload, &payload); err != nil {
		return HostCommandInvocation{}, err
	}
	return payload, nil
}

type PluginCapability string

const (
	PluginCapabilityCommandProvider PluginCapability = "command_provider"
	PluginCapabilityEventSubscriber PluginCapability = "event_subscriber"
	PluginCapabilityUIPanel         PluginCapability = "ui_panel"
)

type PluginHello struct {
	ProtocolVersion int                `json:"protocol_version"`
	PluginID        string             `json:"plugin_id"`
	Name            string             `json:"name"`
	Version         string             `json:"version"`
	Capabilities    []PluginCapability `json:"capabilities,omitempty"`
}

type PluginLogLevel string

const (
	PluginLogLevelTrace PluginLogLevel = "trace"
	PluginLogLevelDebug PluginLogLevel = "debug"
	PluginLogLevelInfo  PluginLogLevel = "info"
	PluginLogLevelWarn  PluginLogLevel = "warn"
	PluginLogLevelError PluginLogLevel = "error"
)

type PluginLogMessage struct {
	Level   PluginLogLevel `json:"level"`
	Message string         `json:"message"`
}

type PluginToastLevel string

const (
	PluginToastLevelInfo    PluginToastLevel = "info"
	PluginToastLevelSuccess PluginToastLevel = "success"
	PluginToastLevelWarning PluginToastLevel = "warning"
	PluginToastLevelError   PluginToastLevel = "error"
)

type PluginToastMessage struct {
	Level      PluginToastLevel `json:"level"`
	Message    string           `json:"message"`
	DurationMS *uint64          `json:"duration_ms,omitempty"`
}

type PluginRPCMessage struct {
	Type    string `json:"type"`
	Payload any    `json:"payload,omitempty"`
}

type PluginMetadata struct {
	PluginID     string
	Name         string
	Version      string
	Capabilities []PluginCapability
}

type PluginSession struct {
	reader    *bufio.Reader
	writer    io.Writer
	HostHello HostHello
	PluginID  string
}

var ErrHostClosedStream = errors.New("host closed the plugin stream")

type ProtocolVersionMismatchError struct {
	Expected int
	Actual   int
}

func (e ProtocolVersionMismatchError) Error() string {
	return fmt.Sprintf("protocol version mismatch: expected %d, got %d", e.Expected, e.Actual)
}

type PluginIDMismatchError struct {
	Expected string
	Actual   string
}

func (e PluginIDMismatchError) Error() string {
	return fmt.Sprintf("plugin id mismatch: expected %q, got %q", e.Expected, e.Actual)
}

type UnexpectedMessageError struct {
	Message string
}

func (e UnexpectedMessageError) Error() string {
	return e.Message
}

func NewStdioSession(metadata PluginMetadata) (*PluginSession, error) {
	return NewSession(bufio.NewReader(os.Stdin), os.Stdout, metadata)
}

func NewSession(reader *bufio.Reader, writer io.Writer, metadata PluginMetadata) (*PluginSession, error) {
	helloMessage, err := ReadHostHello(reader)
	if err != nil {
		return nil, err
	}

	if helloMessage.ProtocolVersion != PluginProtocolVersion {
		return nil, ProtocolVersionMismatchError{
			Expected: PluginProtocolVersion,
			Actual:   helloMessage.ProtocolVersion,
		}
	}

	if helloMessage.PluginID != metadata.PluginID {
		return nil, PluginIDMismatchError{
			Expected: metadata.PluginID,
			Actual:   helloMessage.PluginID,
		}
	}

	session := &PluginSession{
		reader:    reader,
		writer:    writer,
		HostHello: helloMessage,
		PluginID:  helloMessage.PluginID,
	}

	if err := session.Send(NewPluginHelloMessage(PluginHello{
		ProtocolVersion: PluginProtocolVersion,
		PluginID:        helloMessage.PluginID,
		Name:            metadata.Name,
		Version:         metadata.Version,
		Capabilities:    metadata.Capabilities,
	})); err != nil {
		return nil, err
	}

	return session, nil
}

func (s *PluginSession) Recv() (HostRPCMessage, error) {
	return ReadHostMessage(s.reader)
}

func (s *PluginSession) Send(message PluginRPCMessage) error {
	line, err := SerializePluginRPCMessage(message)
	if err != nil {
		return err
	}
	_, err = fmt.Fprintln(s.writer, string(line))
	return err
}

func (s *PluginSession) SendLog(level PluginLogLevel, message string) error {
	return s.Send(NewPluginLogMessage(level, message))
}

func (s *PluginSession) SendPong() error {
	return s.Send(NewPluginPongMessage())
}

func (s *PluginSession) SendToast(level PluginToastLevel, message string, durationMS *uint64) error {
	return s.Send(NewPluginToastMessage(level, message, durationMS))
}

func CommandID(message HostRPCMessage) (string, bool) {
	if message.Type != "invoke_command" {
		return "", false
	}
	payload, err := message.AsInvokeCommand()
	if err != nil {
		return "", false
	}
	return payload.CommandID, true
}

func (s *PluginSession) RunUntilShutdown(
	onMessage func(message HostRPCMessage, session *PluginSession) error,
) error {
	for {
		message, err := s.Recv()
		if err != nil {
			return err
		}

		if err := onMessage(message, s); err != nil {
			return err
		}

		if message.Type == "shutdown" {
			return nil
		}
	}
}

func ReadHostHello(reader *bufio.Reader) (HostHello, error) {
	message, err := ReadHostMessage(reader)
	if err != nil {
		return HostHello{}, err
	}
	if message.Type != "hello" {
		return HostHello{}, UnexpectedMessageError{
			Message: fmt.Sprintf("expected host hello, got %q", message.Type),
		}
	}
	return message.AsHello()
}

func ReadHostMessage(reader *bufio.Reader) (HostRPCMessage, error) {
	line, err := reader.ReadString('\n')
	if err != nil {
		if errors.Is(err, io.EOF) {
			return HostRPCMessage{}, ErrHostClosedStream
		}
		return HostRPCMessage{}, err
	}
	return ParseHostRPCMessage([]byte(line))
}

func ParseHostRPCMessage(line []byte) (HostRPCMessage, error) {
	var message HostRPCMessage
	if err := json.Unmarshal(line, &message); err != nil {
		return HostRPCMessage{}, err
	}
	switch message.Type {
	case "hello", "invoke_command", "shutdown", "ping":
		return message, nil
	default:
		return HostRPCMessage{}, UnexpectedMessageError{
			Message: fmt.Sprintf("unknown host message type %q", message.Type),
		}
	}
}

func ParsePluginRPCMessage(line []byte) (PluginRPCMessage, error) {
	var message PluginRPCMessage
	if err := json.Unmarshal(line, &message); err != nil {
		return PluginRPCMessage{}, err
	}
	switch message.Type {
	case "hello", "log", "toast", "pong":
		return message, nil
	default:
		return PluginRPCMessage{}, UnexpectedMessageError{
			Message: fmt.Sprintf("unknown plugin message type %q", message.Type),
		}
	}
}

func SerializePluginRPCMessage(message PluginRPCMessage) ([]byte, error) {
	return json.Marshal(message)
}

func NewPluginHelloMessage(payload PluginHello) PluginRPCMessage {
	return PluginRPCMessage{
		Type:    "hello",
		Payload: payload,
	}
}

func NewPluginLogMessage(level PluginLogLevel, message string) PluginRPCMessage {
	return PluginRPCMessage{
		Type: "log",
		Payload: PluginLogMessage{
			Level:   level,
			Message: message,
		},
	}
}

func NewPluginToastMessage(level PluginToastLevel, message string, durationMS *uint64) PluginRPCMessage {
	return PluginRPCMessage{
		Type: "toast",
		Payload: PluginToastMessage{
			Level:      level,
			Message:    message,
			DurationMS: durationMS,
		},
	}
}

func NewPluginPongMessage() PluginRPCMessage {
	return PluginRPCMessage{
		Type: "pong",
	}
}
