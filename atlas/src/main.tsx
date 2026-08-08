import { StrictMode } from "react";
import { createRoot } from "react-dom/client";

import { AtlasApp } from "./app/AtlasApp";

const root = document.getElementById("root");

if (!root) {
  throw new Error("Missing root element");
}

createRoot(root).render(
  <StrictMode>
    <AtlasApp />
  </StrictMode>,
);
