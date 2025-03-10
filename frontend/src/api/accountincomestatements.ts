import { useQuery } from "@tanstack/react-query";
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
      const response = await fetch(
        "/api/accountincomestatements/" + accounts_re,
        {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ dates }),
        },
      );

      if (response.status != 200) throw await response.text();

      return await response.json();
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
