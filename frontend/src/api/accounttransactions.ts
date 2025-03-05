import { useQuery } from "@tanstack/react-query";
export function useAccountTransactions(accounts_re: string) {
  return useQuery({
    queryKey: ["accounttransactions", accounts_re],
    queryFn: async (): Promise<Transactions> => {
      const response = await fetch("/api/accounttransactions/" + accounts_re);

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
