// Renders the app icon (public/logo.svg) to the 1024px PNG that `tauri icon`
// turns into every platform icon, and the tray template icon (public/tray.svg).
import { Resvg } from "@resvg/resvg-js";
import { copyFileSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";

const render = (svg, width, out) => {
  const png = new Resvg(readFileSync(new URL(svg, import.meta.url)), {
    fitTo: { mode: "width", value: width },
  }).render().asPng();
  writeFileSync(new URL(out, import.meta.url), png);
  console.log("wrote", out.replace("../", ""));
};

render("../public/logo.svg", 1024, "../src-tauri/icons/source.png");
render("../public/tray.svg", 44, "../src-tauri/icons/tray.png");
// For Appetizer's catalog (it looks for build/icon.png).
mkdirSync(new URL("../build", import.meta.url), { recursive: true });
copyFileSync(new URL("../src-tauri/icons/source.png", import.meta.url), new URL("../build/icon.png", import.meta.url));
