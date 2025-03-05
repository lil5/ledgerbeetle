import { Route, Routes } from "react-router-dom";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";

import { useTheme } from "./components/use-theme";

import IndexPage from "@/pages/index";

function App() {
  const queryClient = new QueryClient();

  useTheme();

  return (
    <QueryClientProvider client={queryClient}>
      <Routes>
        <Route element={<IndexPage />} path="/" />
      </Routes>
    </QueryClientProvider>
  );
}

export default App;
