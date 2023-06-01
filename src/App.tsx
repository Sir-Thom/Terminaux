import { onCleanup, createSignal, createEffect } from "solid-js";
import { Terminal } from "xterm";
import { FitAddon } from "xterm-addon-fit";
import "xterm/css/xterm.css";
import { invoke, tauri, window } from "@tauri-apps/api";
import { Event, listen } from "@tauri-apps/api/event";
import { CanvasAddon } from "xterm-addon-canvas";
import { WebLinksAddon } from "xterm-addon-web-links";

function App() {
  const [terminalElement, setTerminalElement] =
    createSignal<HTMLElement | null>(null);
  const fitAddon = new FitAddon();
  const canvasAddon = new CanvasAddon();

  const term = new Terminal({
    fontFamily: "Monospace",
    fontSize: 16,
    cursorBlink: true,
    cursorStyle: "bar",
    cursorWidth: 2,
    overviewRulerWidth: 0,
    rows: 80,
    cols: 40,
    allowProposedApi: true,
    tabStopWidth: 4,
    smoothScrollDuration: 0,
    scrollback: 1000,

    theme: {
      //cursor color
      cursor: "#ffffff",
      //cursor accent color
      cursorAccent: "#000000",

      //selection color
      selectionInactiveBackground: "#ffffff",
      selectionBackground: "#ffffff",
      // Background color
      background: "rgb(47, 47, 47)",
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

  createEffect(() => {
    const element = terminalElement();
    if (element) {
      term.loadAddon(new WebLinksAddon());
      term.loadAddon(canvasAddon);
      term.loadAddon(fitAddon);
      term.open(element);
    }
    // Make the terminal fit all the window size
    function fitTerminal() {
      fitAddon.fit();
      console.log(term.cols);
      console.log(term.rows);
      void invoke("async_resize_pty", {
        rows: term.rows,
        cols: term.cols,
      });
    }

    // Write data from pty into the terminal
    function writeToTerminal(ev: Event<string>) {
      term.write(ev.payload);
    }

    // Write data from the terminal to the pty
    function writeToPty(data: string) {
      void invoke("async_write_to_pty", {
        data,
      });
    }

    let currentFontSize = term.options.fontSize as number; // Default font size

    // Zoom in the terminal
    function zoomIn() {
      const newFontSize = (currentFontSize += 1);
      term.options.fontSize = newFontSize;
    }

    // Zoom out the terminal
    function zoomOut() {
      const newFontSize = (currentFontSize -= 1);
      term.options.fontSize = newFontSize;
    }

    // Reset the zoom level to default
    function resetZoom() {
      currentFontSize = 16;
      term.options.fontSize = currentFontSize;
    }

    // Add event listeners for zoom shortcuts
    document.addEventListener("keydown", (event) => {
      if (event.ctrlKey && event.key === "+") {
        zoomIn();
      } else if (event.ctrlKey && event.key === "-") {
        zoomOut();
      } else if (event.ctrlKey && event.key === "0") {
        resetZoom();
      }
    });

    term.onData(writeToPty);
    addEventListener("resize", fitTerminal);
    fitTerminal();
    listen("data", writeToTerminal);

    // Cleanup functions
    onCleanup(() => {
      term.dispose();
      removeEventListener("resize", fitTerminal);
    });
  });

  return (
    <div
      id="terminal"
      ref={setTerminalElement}
      class="w-screen h-screen  fixed inline inset-0 "
    ></div>
  );
}

export default App;
