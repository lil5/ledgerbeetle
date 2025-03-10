import { useQuery } from "@tanstack/react-query";

import { getVersion } from "@/client";

export function useVersion() {
  return useQuery({
    queryKey: ["version"],
    queryFn: async (): Promise<string> => {
      const { data, error } = await getVersion({});

      if (error) throw error;

      return data!;
    },
  });
}
