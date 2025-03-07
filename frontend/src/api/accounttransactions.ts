import { useQuery } from "@tanstack/react-query";
export function useAccountTransactions(
  accounts_re: string,
  date_newest: number,
  date_oldest: number,
) {
  return useQuery({
    initialData: [],
    queryKey: ["accounttransactions", accounts_re, date_newest, date_oldest],
    queryFn: async (): Promise<Transactions> => {
      const response = await fetch(
        `/api/accounttransactions/${accounts_re}?date_newest=${date_newest}&date_oldest=${date_oldest}`,
      );

      if (response.status != 200) throw await response.text();

      return await response.json();
    },
  });
}

export type Transactions = Transaction[];

export interface Transaction {
  commodityUnit: string;
  commodityDecimal: number;
  code: number;
  fullDate: number;
  fullDateSubNano: number;
  fullDate2: number;
  relatedId: string;
  transferId: string;
  debitAccount: string;
  creditAccount: string;
  debitAmount: number;
  creditAmount: number;
}
