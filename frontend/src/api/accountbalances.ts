import { useQuery } from "@tanstack/react-query";

import { queryAccountBalances } from "@/client";

export function useAccountBalances(
  accounts_glob: string,
  filterDateIfTrue: null | number = null,
) {
  return useQuery({
    initialData: [],
    queryKey: ["accountbalances", accounts_glob, filterDateIfTrue],
    queryFn: async (): Promise<Balances> => {
      const { data, error } = await queryAccountBalances({
        body: {
          accounts_glob,
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
