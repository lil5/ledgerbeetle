import { useQuery } from "@tanstack/react-query";

import { queryAccountNamesAll } from "@/client";

export function useAccountNames() {
  return useQuery({
    queryKey: ["accountnames"],
    queryFn: async () => {
      const { data, error } = await queryAccountNamesAll({});

      if (error) throw error;

      return data!;
    },
  });
}
