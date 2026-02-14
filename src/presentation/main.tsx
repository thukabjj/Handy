import React from "react";
import ReactDOM from "react-dom/client";
import PresentationMode from "./PresentationMode";
import "@/i18n";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <PresentationMode />
  </React.StrictMode>,
);
