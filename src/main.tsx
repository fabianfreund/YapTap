import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import "./styles/tokens.css";

// One bundle, two windows: the floating pill and the settings window.
const root = createRoot(document.getElementById("root")!);

if (location.hash === "#overlay") {
  import("./overlay/Overlay").then(({ Overlay }) => root.render(<Overlay />));
} else {
  import("./settings/App").then(({ SettingsApp }) =>
    root.render(
      <StrictMode>
        <SettingsApp />
      </StrictMode>,
    ),
  );
}
