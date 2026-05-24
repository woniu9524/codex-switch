import type { Provider, ProviderInput } from "../../lib/types";
import { emptyProviderForm } from "./constants";

export function newProviderForm(activeProviderId?: string | null): ProviderInput {
  return { ...emptyProviderForm, switchNow: !activeProviderId };
}

export function providerToForm(
  provider: Provider,
  activeProviderId?: string | null,
): ProviderInput {
  return {
    id: provider.id,
    name: provider.name,
    endpoint: provider.endpoint,
    apiKey: "",
    model: provider.model,
    homepage: provider.homepage ?? "",
    notes: provider.notes ?? "",
    icon: provider.icon ?? "",
    disableImageGeneration: provider.disableImageGeneration,
    switchNow: provider.id === activeProviderId,
  };
}
