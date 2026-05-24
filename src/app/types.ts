import type { Snapshot } from "../lib/types";

export type View = "control" | "providers" | "provider-form" | "provider-import" | "settings";

export type RunAction = (
  action: () => Promise<Snapshot>,
  success?: string,
) => Promise<Snapshot | null>;

export interface NavController {
  go: (view: View) => void;
  editProvider: (providerId: string) => void;
}
