import { createEffect, createSignal, onCleanup } from "solid-js";
import { ITerminalAddon, Terminal } from "xterm";
import { FitAddon } from "xterm-addon-fit";
import { WebLinksAddon } from "xterm-addon-web-links";
import "xterm/css/xterm.css";
import { invoke } from "@tauri-apps/api";
import { Event, listen } from "@tauri-apps/api/event";

function TerminalScreen() {
  const [terminalElement, setTerminalElement] =
    createSignal<HTMLElement | null>(null);
  const term = new Terminal({
    fontFamily: [
      "Noto Mono for Powerline",
      "Roboto Mono for Powerline",
      "Jetbrains Mono",
      "Menlo",
      "Monaco",
      "Consolas",
      "Liberation Mono",
      "Courier New",
      "Noto Sans Mono CJK SC",
      "Noto Sans Mono CJK TC",
      "Noto Sans Mono CJK KR",
      "Noto Sans Mono CJK JP",
      "Noto Sans Mono CJK HK",
      "Noto Color Emoji",
      "Noto Sans Symbols",
      "monospace",
      "sans-serif",
    ].join(","),
    convertEol: true,
    cursorBlink: true,
    cursorStyle: "bar",
    cursorWidth: 2,
    overviewRulerWidth: 100,
    allowProposedApi: false,
    tabStopWidth: 4,
    smoothScrollDuration: 0,
    scrollback: 80,
    scrollOnUserInput: true,
    scrollSensitivity: 1,

    theme: {
      cursor: "#ffffff", // Cursor color
      cursorAccent: "#000000", // Cursor accent color
      selectionInactiveBackground: "#ffffff", // Inactive selection background color
      selectionBackground: "#ffffff", // Selection background color
      background: "rgb(47, 47, 47)", // Background color
      foreground: "#fff",
      // Text color
      black: "#000000",
      red: "#ff0000",
      green: "#33ff00",
      yellow: "#ffff00",
      blue: "#0066ff",
      magenta: "#cc00ff",
      cyan: "#00ffff",
      // Bright colors
      brightBlack: "#808080",
      brightRed: "#ff0000",
      brightGreen: "#33ff00",
      brightYellow: "#ffff00",
      brightBlue: "#0066ff",
      brightMagenta: "#cc00ff",
      brightCyan: "#00ffff",
      brightWhite: "#ffffff",
    },
  });
  const fitTerminal = async () => {
    try {
      fitAddon.fit();

      await invoke("async_shell");
      await invoke("async_resize_pty", {
        rows: term.rows,
        cols: term.cols,
      });
    } catch (e) {
      console.log(e);
    }
  };
  const fitAddon = new FitAddon();

  createEffect(() => {
    const element = terminalElement();

    if (element) {
      term.loadAddon(new WebLinksAddon());
      term.loadAddon(fitAddon);
      term.open(element);
      fitTerminal();
    }
  });

  const writeToTerminal = (ev: Event<string>) => {
    console.log(ev.payload);
    term.write(ev.payload);
  };

  const writeToPty = (data: string) => {
    void invoke("async_write_to_pty", {
      data,
    });
  };

  onCleanup(() => {
    term.dispose();
    window.removeEventListener("resize", fitTerminal);
  });

  term.onData(writeToPty);
  window.addEventListener("resize", fitTerminal);
  listen("data", writeToTerminal);
  fitTerminal();

  return (
    <div class="flex-grow">
      <div
        id="terminal"
        ref={setTerminalElement}
        class="w-screen h-screen"
      ></div>
    </div>
  );
}

export default TerminalScreen;
