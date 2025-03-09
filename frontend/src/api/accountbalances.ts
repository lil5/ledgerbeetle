import { useQuery } from "@tanstack/react-query";
export function useAccountBalances(
  accounts_re: string,
  filterDateIfTrue: null | number = null,
) {
  return useQuery({
    initialData: [],
    queryKey: ["accountbalances", accounts_re, filterDateIfTrue],
    queryFn: async (): Promise<Balances> => {
      const response = await fetch(
        "/api/accountbalances/" +
          accounts_re +
          (filterDateIfTrue ? `?date=${filterDateIfTrue}` : ""),
      );

      if (response.status != 200) throw await response.text();

      return await response.json();
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
