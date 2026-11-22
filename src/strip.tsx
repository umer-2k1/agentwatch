import React from "react";
import ReactDOM from "react-dom/client";
import { Strip } from "./components/Strip";
import "./index.css";

ReactDOM.createRoot(document.getElementById("strip-root")!).render(
  <React.StrictMode>
    <Strip />
  </React.StrictMode>
);
