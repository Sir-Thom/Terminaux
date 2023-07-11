import { createEffect, createSignal, onCleanup } from "solid-js";
import { ITerminalAddon, ITheme, Terminal } from "xterm";
import { FitAddon } from "xterm-addon-fit";
import { WebLinksAddon } from "xterm-addon-web-links";
import "xterm/css/xterm.css";
import { invoke } from "@tauri-apps/api";
import { Event, listen } from "@tauri-apps/api/event";
import theme from "../data/Kanagawa-Dragon.json";
import { appWindow } from "@tauri-apps/api/window";

function TerminalScreen() {
  const [terminalElement, setTerminalElement] =
    createSignal<HTMLElement | null>(null);
  const xtermTheme: ITheme = {};
  for (const key in theme) {
    if (key !== "name") {
      xtermTheme[key as keyof ITheme] = theme[key];
    }
  }
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
    allowProposedApi: false,
    tabStopWidth: 4,
    smoothScrollDuration: 0,
    scrollback: 80,
    scrollOnUserInput: true,
    scrollSensitivity: 1,
    cols: 80,
    rows: 24,

    theme: xtermTheme,
  });

  const fitTerminal = async () => {
    fitAddon.fit();

    console.log("Resizing terminal");
    console.log(term.rows, term.cols);
    await invoke("async_resize_pty", {
      rows: term.rows,
      cols: term.cols,
    });
  };

  const fitAddon = new FitAddon();

  createEffect(() => {
    const element = terminalElement();

    if (element) {
      term.loadAddon(new WebLinksAddon());
      term.loadAddon(fitAddon);
      term.open(element);

      // Invoke async_shell command to create the shell process
      invoke("async_shell").catch((error) => {
        console.error("Error creating shell:", error);
      });

      fitTerminal();
    }
  });

  const writeToTerminal = (ev: Event<string>) => {
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
  appWindow.emit("resize", fitTerminal);
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
