import { convertFileSrc } from "@tauri-apps/api/core";
import type { Provider } from "../../lib/types";
import { cx } from "../../lib/ui";

export function ProviderAvatar({
  provider,
  size = "lg",
}: {
  provider: Pick<Provider, "name" | "icon">;
  size?: "md" | "lg";
}) {
  const name = provider.name.trim();
  const isDeepSeek = /deepseek/i.test(name);
  const isOpenAI = /openai|gpt/i.test(name);
  const isMoonshot = /moonshot|kimi/i.test(name);
  const iconSrc = provider.icon ? providerIconSrc(provider.icon) : null;

  return (
    <div
      className={cx(
        "provider-avatar grid shrink-0 place-items-center rounded-full border border-stone-200 bg-white font-black shadow-sm",
        size === "lg" ? "size-[48px] text-[15px]" : "size-9 text-[12px]",
        isDeepSeek && "text-blue-600",
        isOpenAI && "text-stone-950",
        isMoonshot && "bg-stone-950 text-white",
      )}
    >
      {iconSrc ? (
        <img className="size-2/3 object-contain" src={iconSrc} alt="" />
      ) : isDeepSeek ? (
        <span className="text-[23px] leading-none">D</span>
      ) : isOpenAI ? (
        <span className="text-[21px] leading-none">◎</span>
      ) : isMoonshot ? (
        <span className="text-[20px] leading-none">M</span>
      ) : (
        <span>{name.slice(0, 2).toUpperCase() || "AI"}</span>
      )}
    </div>
  );
}

function providerIconSrc(icon: string) {
  if (/^(https?:|data:|asset:|http:\/\/asset\.localhost)/i.test(icon)) return icon;
  return convertFileSrc(icon);
}
