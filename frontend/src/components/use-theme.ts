import { useEffect, useState } from "react";

// constant properties for Theme
export const ThemeProps = {
  // light theme
  LIGHT: "light",
  // dark theme
  DARK: "dark",
} as const;

export type Theme = typeof ThemeProps.LIGHT | typeof ThemeProps.DARK;

/**
 * React hook to switch between themes
 *
 * @param defaultTheme the default theme name (e.g. light, dark, purple-dark and etc)
 * @returns An object containing the current theme and theme manipulation functions
 */
export function useTheme() {
  const MEDIA = "(prefers-color-scheme: dark)";

  const [isMounted, setIsMounted] = useState(false);

  const setMediaTheme = (matches?: boolean) => {
    let targetTheme = (
      matches === undefined ? window.matchMedia?.(MEDIA).matches : matches
    )
      ? ThemeProps.DARK
      : ThemeProps.LIGHT;

    const removeTheme =
      targetTheme == ThemeProps.DARK ? ThemeProps.LIGHT : ThemeProps.DARK;

    const elHtml = document.getElementsByTagName("html")[0];

    elHtml.classList.remove(removeTheme);
    elHtml.classList.add(targetTheme);

    return targetTheme;
  };
  const handleMediaQuery = (e: MediaQueryListEvent | MediaQueryList) => {
    setMediaTheme(e.matches);
  };

  useEffect(() => {
    if (!isMounted) return;
    setMediaTheme();
    const media = window.matchMedia(MEDIA);

    media.addEventListener("change", handleMediaQuery);

    return () => media.removeEventListener("change", handleMediaQuery);
  }, [isMounted]);

  useEffect(() => {
    if (!isMounted && window) setIsMounted(true);
  }, [isMounted]);
}
