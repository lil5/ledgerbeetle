import { useQuery } from "@tanstack/react-query";

import { getAccountBalances } from "@/client";

export function useAccountBalances(
  accounts_re: string,
  filterDateIfTrue: null | number = null,
) {
  return useQuery({
    initialData: [],
    queryKey: ["accountbalances", accounts_re, filterDateIfTrue],
    queryFn: async (): Promise<Balances> => {
      const { data, error } = await getAccountBalances({
        path: {
          filter: accounts_re,
        },
        query: {
          date: filterDateIfTrue ? filterDateIfTrue : undefined,
        },
      });

      if (error) throw error;

      return data!;
    },
  });
}

export type Balances = Balance[];

export interface Balance {
  accountName: string;
  amount: number;
  commodityUnit: string;
  commodityDecimal: number;
}
