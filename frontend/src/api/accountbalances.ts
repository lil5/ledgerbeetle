import { useQuery } from "@tanstack/react-query";
export function useAccountBalances(accounts_re: string) {
  return useQuery({
    queryKey: ["accountbalances", accounts_re],
    queryFn: async (): Promise<Balances> => {
      const response = await fetch("/api/accountbalances/" + accounts_re);

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
