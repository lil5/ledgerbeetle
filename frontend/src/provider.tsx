import type { NavigateOptions } from "react-router-dom";

import { I18nProvider } from "@react-aria/i18n";
import { HeroUIProvider } from "@heroui/system";
import { useHref, useNavigate } from "react-router-dom";

declare module "@react-types/shared" {
  interface RouterConfig {
    routerOptions: NavigateOptions;
  }
}

export function Provider({ children }: { children: React.ReactNode }) {
  const navigate = useNavigate();

  return (
    <HeroUIProvider navigate={navigate} useHref={useHref}>
      <I18nProvider locale="nl-NL">{children}</I18nProvider>
    </HeroUIProvider>
  );
}
