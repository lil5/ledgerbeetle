import { useMutation, useQueryClient } from "@tanstack/react-query";

import { AddTransactions, putAdd } from "@/client";

export function useAddTransactions() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationKey: ["add"],
    mutationFn: async (
      add_transactions: AddTransactions,
    ): Promise<string[]> => {
      const { data, error } = await putAdd({
        body: add_transactions,
      });

      if (error) throw error;

      return data!;
    },
    onSuccess: () => {
      // Invalidate and refetch
      queryClient.invalidateQueries();
    },
  });
}
