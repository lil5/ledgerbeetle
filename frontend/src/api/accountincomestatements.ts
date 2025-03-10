import { useQuery } from "@tanstack/react-query";

import { getAccountIncomeStatement } from "@/client";

export function useAccountIncomeStatements(
  accounts_re: string,
  dates: number[],
) {
  return useQuery({
    initialData: {
      dates: [],
      incomeStatements: [],
    },
    queryKey: ["accountincomestatements", accounts_re, dates.join(",")],
    queryFn: async (): Promise<IncomeStatements> => {
      const { data, error } = await getAccountIncomeStatement({
        path: { filter: accounts_re },
        body: { dates },
      });

      if (error) throw error;

      return data!;
    },
  });
}

export interface IncomeStatements {
  dates: number[];
  incomeStatements: IncomeStatement[];
}

export interface IncomeStatement {
  accountName: string;
  amounts: number[];
  commodityUnit: string;
  commodityDecimal: number;
}
