import { getCurrentWindow } from "@tauri-apps/api/window";

export async function controlWindow(action: "minimize" | "maximize" | "close") {
  const currentWindow = getCurrentWindow();
  if (action === "minimize") await currentWindow.minimize();
  if (action === "maximize") await currentWindow.toggleMaximize();
  if (action === "close") await currentWindow.close();
}
