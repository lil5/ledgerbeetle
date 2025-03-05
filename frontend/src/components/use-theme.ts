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

  const setMediaTheme = () => {
    if (!(window && document)) return ThemeProps.LIGHT;

    const targetTheme = window.matchMedia?.(MEDIA).matches
      ? ThemeProps.DARK
      : ThemeProps.LIGHT;

    const removeTheme =
      targetTheme == ThemeProps.DARK ? ThemeProps.LIGHT : ThemeProps.DARK;

    document.documentElement.classList.remove(removeTheme);
    document.documentElement.classList.add(targetTheme);

    return targetTheme;
  };
  const [theme, setTheme] = useState<Theme>(ThemeProps.LIGHT);

  const handleMediaQuery = (_: MediaQueryListEvent | MediaQueryList) => {
    setTheme(setMediaTheme());
  };

  useEffect(() => {
    if (!isMounted) return;
    const media = window.matchMedia(MEDIA);

    media.addEventListener("change", handleMediaQuery);

    return () => media.removeEventListener("change", handleMediaQuery);
  }, [isMounted]);

  useEffect(() => {
    setIsMounted(true);
  }, [isMounted]);

  return theme;
}
