import React from "react";
import { Link, Plus, Search } from "lucide-react";
import type { RunAction, View } from "../app/types";
import { Button, EmptyState } from "../components/ui";
import { Page } from "../components/layout/Page";
import { ProviderCard } from "../features/providers/ProviderCard";
import { api } from "../lib/api";
import type { Snapshot } from "../lib/types";
import { useI18n } from "../i18n";

export function ProvidersPage({
  snapshot,
  busy,
  run,
  go,
  editProvider,
}: {
  snapshot: Snapshot;
  busy: boolean;
  run: RunAction;
  go: (view: View) => void;
  editProvider: (providerId: string) => void;
}) {
  const [query, setQuery] = React.useState("");
  const { t } = useI18n();
  const providers = snapshot.state.providers.filter((provider) => {
    const value = `${provider.name} ${provider.endpoint} ${provider.model}`.toLowerCase();
    return value.includes(query.trim().toLowerCase());
  });

  return (
    <Page
      title={t("providers.title")}
      actions={
        <Button size="lg" tone="secondary" onClick={() => go("provider-form")}>
          <Plus size={17} />
          {t("providers.add")}
        </Button>
      }
    >
      <label className="relative block">
        <Search
          className="pointer-events-none absolute left-5 top-1/2 -translate-y-1/2 text-stone-500"
          size={20}
        />
        <input
          className="h-10 w-full rounded-md border border-stone-300 bg-white pl-12 pr-3 text-[14px] outline-none transition placeholder:text-stone-400 focus:border-emerald-600 focus:ring-3 focus:ring-emerald-100"
          value={query}
          onChange={(event) => setQuery(event.target.value)}
          placeholder={t("providers.searchPlaceholder")}
        />
      </label>

      {snapshot.state.providers.length === 0 ? (
        <EmptyState title={t("providers.emptyTitle")} action={t("providers.emptyAction")} />
      ) : providers.length === 0 ? (
        <EmptyState title={t("providers.noMatchTitle")} action={t("providers.noMatchAction")} />
      ) : (
        <div className="flex flex-col gap-2.5">
          {providers.map((provider) => (
            <ProviderCard
              key={provider.id}
              provider={provider}
              isCurrent={provider.id === snapshot.state.activeProviderId}
              busy={busy}
              onSwitch={() => run(() => api.switchProvider(provider.id), t("providers.switched"))}
              onEdit={() => editProvider(provider.id)}
              onDelete={() => run(() => api.deleteProvider(provider.id), t("providers.deleted"))}
            />
          ))}
        </div>
      )}

      <Button className="mt-auto w-full" size="lg" onClick={() => go("provider-import")}>
        <Link size={17} />
        {t("providers.importLink")}
      </Button>
    </Page>
  );
}
