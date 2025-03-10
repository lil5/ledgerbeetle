import { useQuery } from "@tanstack/react-query";

import { getAccountNames } from "@/client";

export function useAccountNames() {
  return useQuery({
    queryKey: ["accountnames"],
    queryFn: async (): Promise<Array<string>> => {
      const { data, error } = await getAccountNames({});

      if (error) throw error;

      return data!;
    },
  });
}
