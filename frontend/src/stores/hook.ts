import { AnyUpdater, useStore, type Store } from "@tanstack/react-store";

export default function useStoreHook<TState, TUpdater extends AnyUpdater>(
  store: Store<TState, TUpdater>,
): [TState, (updater: TUpdater) => void] {
  const state = useStore(store);
  const setState = store.setState;

  return [state, setState];
}
