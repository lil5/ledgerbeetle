import { useQuery } from "@tanstack/react-query";

import { queryAccountTransactions, Transaction } from "@/client";

export function useAccountTransactions(
  accounts_glob: string,
  date_newest: number,
  date_oldest: number,
) {
  return useQuery({
    initialData: [],
    queryKey: ["accounttransactions", accounts_glob, date_newest, date_oldest],
    queryFn: async (): Promise<Transaction[]> => {
      const { data, error } = await queryAccountTransactions({
        body: { date_newest, date_oldest, accounts_glob },
      });

      if (error) throw error;

      return data!;
    },
  });
}
