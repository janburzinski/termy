import { useState, useRef, useEffect, useCallback } from "react";

const ASCII_ART = [
  "                  ..'",
  "               ,xNMM.",
  "             .OMMMMo",
  '             lMM"',
  "   .;loddo:. .olloddol;.",
  " cKMMMMMMMMMMNWMMMMMMMMMM0:",
  ".KMMMMMMMMMMMMMMMMMMMMMMMWd.",
  "XMMMMMMMMMMMMMMMMMMMMMMMX.",
  ";MMMMMMMMMMMMMMMMMMMMMMMM:",
  ":MMMMMMMMMMMMMMMMMMMMMMMM:",
  ".MMMMMMMMMMMMMMMMMMMMMMMMX.",
  " kMMMMMMMMMMMMMMMMMMMMMMMMWd.",
  " 'XMMMMMMMMMMMMMMMMMMMMMMMMMMk",
  "  'XMMMMMMMMMMMMMMMMMMMMMMMMK.",
  "    kMMMMMMMMMMMMMMMMMMMMMMd",
  '     ;KMMMMMMMWXXWMMMMMMMk.',
  '       "cooc*"    "*coo\'"',
];

const SYSTEM_INFO = [
  ["", "lassevestergaard@Mac"],
  ["", "--------------------"],
  ["OS:", "macOS Tahoe 26.2 (25C56) arm64"],
  ["Host:", "MacBook Pro (14-inch, 2024, Three Thunderbolt 4 ports)"],
  ["Kernel:", "Darwin 25.2.0"],
  ["Uptime:", "20 hours, 31 mins"],
  ["Packages:", "283 (brew), 49 (brew-cask)"],
  ["Shell:", "zsh 5.9"],
  ["Display (Color LCD):", "3024x1964 @ 2x in 14\", 120 Hz [Built-in]"],
  ["WM:", "Quartz Compositor 1.600.0"],
  ["WM Theme:", "Multicolor (Dark)"],
  ["Theme:", "Liquid Glass"],
  ["Font:", ".AppleSystemUIFont [System], Helvetica [User]"],
  ["Cursor:", "Fill - Black, Outline - White (32px)"],
  ["Terminal:", "termy"],
  ["CPU:", "Apple M4 (10) @ 4.46 GHz"],
  ["GPU:", "Apple M4 (10) @ 1.58 GHz [Integrated]"],
  ["Memory:", "16.01 GB / 24.00 GB (67%)"],
  ["Swap:", "Disabled"],
  ["Disk (/):", "166.67 GB / 460.43 GB (36%) - apfs [Read-only]"],
  ["Local IP (en0):", "192.168.8.8/24"],
  ["Battery (bq40z651):", "51% (5 hours, 6 mins remaining) [Discharging]"],
  ["Locale:", "C.UTF-8"],
];

const COLOR_BLOCKS = [
  ["#2e3436", "#cc0000", "#4e9a06", "#c4a000", "#3465a4", "#75507b", "#06989a", "#d3d7cf"],
  ["#555753", "#ef2929", "#8ae234", "#fce94f", "#729fcf", "#ad7fa8", "#34e2e2", "#eeeeec"],
];

interface TerminalLine {
  id: number;
  type: "prompt" | "output" | "fastfetch";
  content?: string;
  command?: string;
}

let nextLineId = 0;

const KNOWN_COMMANDS: Record<string, string[]> = {
  help: [
    "Available commands:",
    "  fastfetch  - Display system information",
    "  help       - Show this help message",
    "  clear      - Clear the terminal",
    "  whoami     - Display current user",
    "  echo       - Echo a message",
    "  neofetch   - Alias for fastfetch",
    "  ls         - List files",
    "",
    "This is a demo terminal. Commands are simulated.",
  ],
  whoami: ["dev"],
  ls: [
    "Documents  Downloads  Desktop  Music  Pictures  Videos",
    ".config    .zshrc     .gitconfig",
  ],
  neofetch: ["fastfetch"],
  pwd: ["/home/dev"],
  date: [],
  uname: ["Darwin"],
};

function FastfetchOutput() {
  const maxAsciiWidth = 40;

  return (
    <div className="flex gap-2">
      {/* ASCII Art */}
      <div className="shrink-0 text-[#4ade80]">
        {ASCII_ART.map((line, i) => (
          <div key={i} className="leading-[1.25]">
            <span className="whitespace-pre">{line.padEnd(maxAsciiWidth)}</span>
          </div>
        ))}
      </div>
      {/* System Info */}
      <div className="min-w-0">
        {SYSTEM_INFO.map(([label, value], i) => (
          <div key={i} className="leading-[1.25] whitespace-nowrap">
            {label ? (
              <>
                <span className="text-[#4ade80] font-bold">{label}</span>{" "}
                <span className="text-[#d1d5db]">{value}</span>
              </>
            ) : (
              <span className="text-[#d1d5db]">{value}</span>
            )}
          </div>
        ))}
        {/* Color blocks */}
        <div className="mt-2 flex flex-col">
          {COLOR_BLOCKS.map((row, ri) => (
            <div key={ri} className="flex">
              {row.map((color, ci) => (
                <span
                  key={ci}
                  className="inline-block w-[30px] h-[18px]"
                  style={{ backgroundColor: color }}
                />
              ))}
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}

function PromptPrefix() {
  return (
    <span className="whitespace-pre shrink-0">
      <span className="text-[#4ade80]">→</span>
      {"  "}
      <span className="text-[#82aaff]">dev</span>
      {" "}
    </span>
  );
}

export function InteractiveTerminal() {
  const [lines, setLines] = useState<TerminalLine[]>([
    { id: nextLineId++, type: "prompt", command: "fastfetch" },
    { id: nextLineId++, type: "fastfetch" },
  ]);
  const [currentInput, setCurrentInput] = useState("");
  const [commandHistory, setCommandHistory] = useState<string[]>(["fastfetch"]);
  const [historyIndex, setHistoryIndex] = useState(-1);
  const inputRef = useRef<HTMLInputElement>(null);
  const terminalRef = useRef<HTMLDivElement>(null);
  const [isFocused, setIsFocused] = useState(false);

  const scrollToBottom = useCallback(() => {
    if (terminalRef.current) {
      terminalRef.current.scrollTop = terminalRef.current.scrollHeight;
    }
  }, []);

  useEffect(() => {
    scrollToBottom();
  }, [lines, scrollToBottom]);

  const processCommand = useCallback((cmd: string) => {
    const trimmed = cmd.trim();

    if (trimmed === "clear") {
      setLines([]);
      return;
    }

    setLines((prev) => {
      const newLines: TerminalLine[] = [
        ...prev,
        { id: nextLineId++, type: "prompt", command: trimmed },
      ];

      if (!trimmed) return newLines;

      if (trimmed === "fastfetch" || trimmed === "neofetch") {
        newLines.push({ id: nextLineId++, type: "fastfetch" });
        return newLines;
      }

      if (trimmed.startsWith("echo ")) {
        newLines.push({ id: nextLineId++, type: "output", content: trimmed.slice(5) });
        return newLines;
      }

      if (trimmed === "date") {
        newLines.push({ id: nextLineId++, type: "output", content: new Date().toString() });
        return newLines;
      }

      const output = KNOWN_COMMANDS[trimmed];
      if (output) {
        if (output[0] === "fastfetch") {
          newLines.push({ id: nextLineId++, type: "fastfetch" });
        } else {
          for (const line of output) {
            newLines.push({ id: nextLineId++, type: "output", content: line });
          }
        }
        return newLines;
      }

      newLines.push({
        id: nextLineId++,
        type: "output",
        content: `zsh: command not found: ${trimmed}`,
      });
      return newLines;
    });
  }, []);

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent<HTMLInputElement>) => {
      if (e.key === "Enter") {
        e.preventDefault();
        const cmd = inputRef.current?.value ?? "";
        if (cmd.trim()) {
          setCommandHistory((prev) => [...prev, cmd.trim()]);
        }
        setHistoryIndex(-1);
        processCommand(cmd);
        setCurrentInput("");
      } else if (e.key === "ArrowUp") {
        e.preventDefault();
        setCommandHistory((prev) => {
          setHistoryIndex((hi) => {
            const newIndex =
              hi === -1 ? prev.length - 1 : Math.max(0, hi - 1);
            if (prev[newIndex]) {
              setCurrentInput(prev[newIndex]);
            }
            return newIndex;
          });
          return prev;
        });
      } else if (e.key === "ArrowDown") {
        e.preventDefault();
        setCommandHistory((prev) => {
          setHistoryIndex((hi) => {
            if (hi === -1) return -1;
            const newIndex = hi + 1;
            if (newIndex >= prev.length) {
              setCurrentInput("");
              return -1;
            }
            setCurrentInput(prev[newIndex]);
            return newIndex;
          });
          return prev;
        });
      } else if (e.key === "l" && e.ctrlKey) {
        e.preventDefault();
        setLines([]);
      }
    },
    [processCommand],
  );

  const focusInput = useCallback(() => {
    inputRef.current?.focus();
  }, []);

  return (
    <div
      className="terminal-body select-text cursor-text"
      ref={terminalRef}
      onClick={focusInput}
    >
      {/* Rendered lines */}
      {lines.map((line) => {
        if (line.type === "fastfetch") {
          return (
            <div key={line.id} className="mb-3">
              <FastfetchOutput />
            </div>
          );
        }
        if (line.type === "prompt") {
          return (
            <div key={line.id} className="flex">
              <PromptPrefix />
              <span className="text-[#d1d5db]">{line.command}</span>
            </div>
          );
        }
        return (
          <div key={line.id} className="text-[#d1d5db]">
            {line.content || "\u00A0"}
          </div>
        );
      })}

      {/* Active input line */}
      <div className="flex items-center">
        <PromptPrefix />
        <div className="relative flex-1">
          <input
            ref={inputRef}
            type="text"
            value={currentInput}
            onChange={(e) => setCurrentInput(e.target.value)}
            onKeyDown={handleKeyDown}
            onFocus={() => setIsFocused(true)}
            onBlur={() => setIsFocused(false)}
            className="terminal-input"
            spellCheck={false}
            autoComplete="off"
            autoCapitalize="off"
            autoFocus={false}
          />
          {/* Custom block cursor */}
          <span
            className="absolute top-0 left-0 pointer-events-none text-transparent"
            style={{ fontSize: "inherit", lineHeight: "inherit" }}
          >
            {currentInput}
            <span
              className={`inline-block w-[8px] h-[15px] translate-y-[1px] ${
                isFocused
                  ? "bg-[#d1d5db] animate-terminal-blink"
                  : "bg-[#d1d5db]/40"
              }`}
            />
          </span>
        </div>
      </div>
    </div>
  );
}
