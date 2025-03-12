import { useMutation, useQueryClient } from "@tanstack/react-query";

import {
  AddTransactions,
  AddFilterTransactions,
  mutateAdd,
  queryPrepareAddFcfs,
} from "@/client";

export function useAddTransactions() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationKey: ["add"],
    mutationFn: async (
      add_transactions: AddTransactions,
    ): Promise<string[]> => {
      const { data, error } = await mutateAdd({
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

export function useAddTransactionsGlob() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationKey: ["add"],
    mutationFn: async (
      add_filter_transactions: AddFilterTransactions,
    ): Promise<string[]> => {
      let add_transactions: AddTransactions;

      {
        const { data, error } = await queryPrepareAddFcfs({
          body: add_filter_transactions,
        });

        if (error) throw error;
        add_transactions = data!;
      }

      const { data, error } = await mutateAdd({
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
