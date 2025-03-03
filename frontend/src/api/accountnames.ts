import { useQuery } from "@tanstack/react-query";
export function useAccountNames() {
  return useQuery({
    queryKey: ["accountnames"],
    queryFn: async (): Promise<Array<string>> => {
      const response = await fetch("/accountnames");

      return await response.json();
    },
  });
}
