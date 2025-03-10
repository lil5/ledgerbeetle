import { useQuery } from "@tanstack/react-query";

import { getTransactions, Transaction } from "@/client";

export function useAccountTransactions(
  accounts_re: string,
  date_newest: number,
  date_oldest: number,
) {
  return useQuery({
    initialData: [],
    queryKey: ["accounttransactions", accounts_re, date_newest, date_oldest],
    queryFn: async (): Promise<Transaction[]> => {
      const { data, error } = await getTransactions({
        path: { filter: accounts_re },
        query: { date_newest, date_oldest },
      });

      if (error) throw error;

      return data!;
    },
  });
}
