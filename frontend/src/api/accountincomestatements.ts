import { useQuery } from "@tanstack/react-query";

import { queryAccountIncomeStatement } from "@/client";

export function useAccountIncomeStatements(
  accounts_glob: string,
  dates: number[],
) {
  return useQuery({
    initialData: {
      dates: [],
      incomeStatements: [],
    },
    queryKey: ["accountincomestatements", accounts_glob, dates.join(",")],
    queryFn: async () => {
      if (accounts_glob == "") return { dates: [], incomeStatements: [] };
      const { data, error } = await queryAccountIncomeStatement({
        body: { dates, accounts_glob },
      });

      if (error) throw error;

      return data!;
    },
  });
}
