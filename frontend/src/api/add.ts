import { useMutation, useQueryClient } from "@tanstack/react-query";
export function useAddTransactions() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationKey: ["add"],
    mutationFn: async (
      add_transactions: AddTransactions,
    ): Promise<string[]> => {
      const response = await fetch("/api/add", {
        method: "PUT",
        headers: {
          "content-type": "application/json",
        },
        body: JSON.stringify(add_transactions),
      });

      return await response.json();
    },
    onSuccess: () => {
      // Invalidate and refetch
      queryClient.invalidateQueries();
    },
  });
}

export interface AddTransactions {
  fullDate2: number;
  transactions: AddTransaction[];
}

export interface AddTransaction {
  code: number;
  commodityUnit: string;
  relatedId: string;
  debitAccount: string;
  creditAccount: string;
  amount: number;
}
