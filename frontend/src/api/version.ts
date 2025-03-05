import { useQuery } from "@tanstack/react-query";
export function useVersion() {
  return useQuery({
    queryKey: ["version"],
    queryFn: async (): Promise<string> => {
      const response = await fetch("/api/version");

      if (response.status != 200) throw await response.text();

      return await response.json();
    },
  });
}
